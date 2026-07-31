use super::image::QR_CAPACITY;
use crate::envelope;
use crate::envelope::Envelope;
use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

type FormatVersion = u8;
type FragmentIndex = u16;
type FragmentTotal = u16;
type Sha256Type = [u8; SHA_256_SIZE];

const FORMAT_VERSION_SIZE: usize = size_of::<FormatVersion>();
const FRAGMENT_INDEX_SIZE: usize = size_of::<FragmentIndex>();
const FRAGMENT_TOTAL_SIZE: usize = size_of::<FragmentTotal>();
const SHA_256_SIZE: usize = 32;
const SHA_256_SHIFT: usize = FORMAT_VERSION_SIZE + FRAGMENT_INDEX_SIZE + FRAGMENT_TOTAL_SIZE;
const HEADER_SIZE: usize =
    FORMAT_VERSION_SIZE + FRAGMENT_INDEX_SIZE + FRAGMENT_TOTAL_SIZE + SHA_256_SIZE;
const MAX_FRAGMENT_PAYLOAD_SIZE: usize = QR_CAPACITY - HEADER_SIZE;
const FORMAT_VERSION: FormatVersion = 0x01;

#[derive(Debug, thiserror::Error)]
pub enum FragmentDeserializeError {
    #[error("Invalid fragment header: expected at least {HEADER_SIZE} bytes, got {0}")]
    InvalidHeader(usize),
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u8),
    #[error("Fragment too large: {0} bytes, maximum is {MAX_FRAGMENT_PAYLOAD_SIZE}")]
    FragmentTooLarge(usize),
}

pub(crate) struct Fragment {
    index: FragmentIndex,
    total: FragmentTotal,
    sha256: Sha256Type,
    data: Box<[u8]>,
}

impl Fragment {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.data.len());
        buf.push(FORMAT_VERSION);
        buf.extend_from_slice(&self.index.to_be_bytes());
        buf.extend_from_slice(&self.total.to_be_bytes());
        buf.extend_from_slice(&self.sha256);
        buf.extend_from_slice(&self.data);
        buf
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Fragment, FragmentDeserializeError> {
        if bytes.len() < HEADER_SIZE {
            return Err(FragmentDeserializeError::InvalidHeader(bytes.len()));
        }
        let version = bytes[0];
        if version != FORMAT_VERSION {
            return Err(FragmentDeserializeError::UnsupportedVersion(version));
        }
        let index = FragmentIndex::from_be_bytes([bytes[1], bytes[2]]);
        let total = FragmentTotal::from_be_bytes([bytes[3], bytes[4]]);
        let mut sha256 = [0u8; SHA_256_SIZE];
        sha256.copy_from_slice(&bytes[SHA_256_SHIFT..HEADER_SIZE]);
        let data = bytes[HEADER_SIZE..].to_vec();
        if data.len() > MAX_FRAGMENT_PAYLOAD_SIZE {
            return Err(FragmentDeserializeError::FragmentTooLarge(data.len()));
        }
        Ok(Fragment {
            index,
            total,
            sha256,
            data: data.into_boxed_slice(),
        })
    }
}

pub(crate) struct FragmentedEnvelope {
    header: Box<[Fragment]>,
    ciphertext: Box<[Fragment]>,
}

#[derive(Debug, thiserror::Error)]
pub enum EnvelopeFragmentationError {
    #[error("Data too large: requires {0} fragments, maximum is {1}")]
    TooManyFragments(usize, usize),
    #[error(transparent)]
    SerializeHeader(#[from] envelope::binary::SerializeError),
}

impl FragmentedEnvelope {
    pub(crate) fn from_envelope(envelope: &Envelope) -> Result<Self, EnvelopeFragmentationError> {
        let header_bytes = envelope::binary::serialize_header(envelope)?;
        let ciphertext_bytes = &envelope.ciphertext;
        let header_count = header_bytes.len().div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE);
        let ciphertext_count = ciphertext_bytes.len().div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE);
        let total_fragments = header_count + ciphertext_count;
        if total_fragments > FragmentTotal::MAX as usize {
            return Err(EnvelopeFragmentationError::TooManyFragments(
                total_fragments,
                FragmentTotal::MAX as usize,
            ));
        }
        let total = total_fragments as FragmentTotal;
        let mut hasher = Sha256::new();
        hasher.update(&header_bytes);
        hasher.update(ciphertext_bytes);
        let sha256: Sha256Type = hasher.finalize().into();
        let make_fragments =
            |chunks: std::slice::Chunks<'_, u8>, index_offset: usize| -> Box<[Fragment]> {
                chunks
                    .enumerate()
                    .map(|(i, chunk)| Fragment {
                        index: (index_offset + i) as FragmentIndex + 1,
                        total,
                        sha256,
                        data: chunk.to_vec().into_boxed_slice(),
                    })
                    .collect()
            };
        let header = make_fragments(header_bytes.chunks(MAX_FRAGMENT_PAYLOAD_SIZE), 0);
        let ciphertext = make_fragments(
            ciphertext_bytes.chunks(MAX_FRAGMENT_PAYLOAD_SIZE),
            header_count,
        );
        Ok(FragmentedEnvelope { header, ciphertext })
    }

    pub(crate) fn fragments(&self) -> impl Iterator<Item = &Fragment> {
        self.header.iter().chain(self.ciphertext.iter())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssembleFragmentsError {
    #[error("Missing fragments: {0:?}")]
    MissingFragments(Vec<u16>),
    #[error("Conflicting SHA-256 across fragments")]
    ConflictingSha256,
    #[error("Duplicate fragment {0} with different data")]
    ConflictingDuplicate(u16),
    #[error("SHA-256 mismatch: expected {expected}, got {actual}")]
    Sha256Mismatch { expected: String, actual: String },
}

pub(crate) fn assemble_fragments(
    mut fragments: Vec<Fragment>,
) -> Result<Vec<u8>, AssembleFragmentsError> {
    if fragments.is_empty() {
        return Err(AssembleFragmentsError::MissingFragments(vec![1]));
    }
    let expected_total = fragments[0].total;
    let expected_sha256 = fragments[0].sha256;
    for fragment in &fragments {
        if fragment.sha256 != expected_sha256 {
            return Err(AssembleFragmentsError::ConflictingSha256);
        }
    }
    fragments.sort_by_key(|f| f.index);
    let mut deduped: Vec<Fragment> = Vec::with_capacity(expected_total as usize);
    for fragment in fragments {
        if let Some(last) = deduped.last()
            && last.index == fragment.index
        {
            if last.data != fragment.data {
                return Err(AssembleFragmentsError::ConflictingDuplicate(fragment.index));
            }
            continue;
        }
        deduped.push(fragment);
    }
    let mut missing: Vec<u16> = Vec::new();
    let mut deduped_iter = deduped.iter();
    let mut current = deduped_iter.next();
    for expected in 1..=expected_total {
        if current.is_some_and(|f| f.index == expected) {
            current = deduped_iter.next();
        } else {
            missing.push(expected);
        }
    }
    if !missing.is_empty() {
        return Err(AssembleFragmentsError::MissingFragments(missing));
    }
    let assembled: Vec<u8> = deduped
        .into_iter()
        .flat_map(|f| f.data.into_vec())
        .collect();
    let actual_sha256: Sha256Type = Sha256::digest(&assembled).into();
    if actual_sha256 != expected_sha256 {
        return Err(AssembleFragmentsError::Sha256Mismatch {
            expected: HEXLOWER.encode(&expected_sha256),
            actual: HEXLOWER.encode(&actual_sha256),
        });
    }
    Ok(assembled)
}

pub(crate) fn deserialize_fragments(
    payloads: &[Vec<u8>],
) -> Result<Vec<Fragment>, FragmentDeserializeError> {
    payloads
        .iter()
        .map(|payload| Fragment::from_bytes(payload))
        .collect()
}
