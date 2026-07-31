pub(crate) mod image;
pub(crate) mod tar;

use crate::envelope;
use ::image::{DynamicImage, ImageError, ImageFormat, load_from_memory};
use image::{QR_CAPACITY, decode_images, encode_image};
use qrcode::types::QrError;
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
const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8];

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
    pub version: FormatVersion,
    pub index: FragmentIndex,
    pub total: FragmentTotal,
    pub sha256: Sha256Type,
    pub data: Vec<u8>,
}

impl Fragment {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.data.len());
        buf.push(self.version);
        buf.extend_from_slice(&self.index.to_be_bytes());
        buf.extend_from_slice(&self.total.to_be_bytes());
        buf.extend_from_slice(&self.sha256);
        buf.extend_from_slice(&self.data);
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Fragment, FragmentDeserializeError> {
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
            version,
            index,
            total,
            sha256,
            data,
        })
    }
}

struct EnvelopeFragments {
    pub header: Vec<Fragment>,
    pub ciphertext: Vec<Fragment>,
}

#[derive(Debug, thiserror::Error)]
pub enum SplitIntoFragmentsError {
    #[error("Data too large: requires {0} fragments, maximum is {1}")]
    TooManyFragments(usize, usize),
    #[error(transparent)]
    SerializeHeader(#[from] envelope::binary::SerializeError),
}

fn split_into_fragments(
    envelope: &envelope::Envelope,
) -> Result<EnvelopeFragments, SplitIntoFragmentsError> {
    let header_bytes = envelope::binary::serialize_header(envelope)?;
    let ciphertext_bytes = &envelope.ciphertext;
    let header_count = header_bytes.len().div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE);
    let ciphertext_count = ciphertext_bytes.len().div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE);
    let total_fragments = header_count + ciphertext_count;
    if total_fragments > FragmentTotal::MAX as usize {
        return Err(SplitIntoFragmentsError::TooManyFragments(
            total_fragments,
            FragmentTotal::MAX as usize,
        ));
    }
    let total = total_fragments as FragmentTotal;
    let mut full = Vec::with_capacity(header_bytes.len() + ciphertext_bytes.len());
    full.extend_from_slice(&header_bytes);
    full.extend_from_slice(ciphertext_bytes);
    let sha256: Sha256Type = Sha256::digest(&full).into();
    let make_fragments =
        |chunks: std::slice::Chunks<'_, u8>, index_offset: usize| -> Vec<Fragment> {
            chunks
                .enumerate()
                .map(|(i, chunk)| Fragment {
                    version: FORMAT_VERSION,
                    index: (index_offset + i) as FragmentIndex + 1,
                    total,
                    sha256,
                    data: chunk.to_vec(),
                })
                .collect()
        };
    let header = make_fragments(header_bytes.chunks(MAX_FRAGMENT_PAYLOAD_SIZE), 0);
    let ciphertext = make_fragments(
        ciphertext_bytes.chunks(MAX_FRAGMENT_PAYLOAD_SIZE),
        header_count,
    );
    Ok(EnvelopeFragments { header, ciphertext })
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

fn assemble_fragments(mut fragments: Vec<Fragment>) -> Result<Vec<u8>, AssembleFragmentsError> {
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
    let assembled: Vec<u8> = deduped.into_iter().flat_map(|f| f.data).collect();
    let actual_sha256: Sha256Type = Sha256::digest(&assembled).into();
    if actual_sha256 != expected_sha256 {
        fn hex(bytes: &[u8]) -> String {
            bytes.iter().map(|b| format!("{b:02x}")).collect()
        }
        return Err(AssembleFragmentsError::Sha256Mismatch {
            expected: hex(&expected_sha256),
            actual: hex(&actual_sha256),
        });
    }
    Ok(assembled)
}

#[derive(Debug, thiserror::Error)]
pub enum EncodePngFilesError {
    #[error(transparent)]
    Write(#[from] ImageError),
}

fn encode_png_files(images: &[DynamicImage]) -> Result<Vec<tar::File>, EncodePngFilesError> {
    images
        .iter()
        .enumerate()
        .map(|(i, image)| {
            let mut content = Vec::new();
            image.write_to(&mut std::io::Cursor::new(&mut content), ImageFormat::Png)?;
            Ok(tar::File {
                path: format!("{:05}.png", i + 1),
                content,
            })
        })
        .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractImagesError {
    #[error(transparent)]
    LoadImage(#[from] ImageError),
    #[error(transparent)]
    UnpackTar(#[from] tar::UnpackError),
}

fn extract_images(data: &[u8]) -> Result<Vec<DynamicImage>, ExtractImagesError> {
    let images: Vec<DynamicImage> = if data.starts_with(PNG_MAGIC) || data.starts_with(JPEG_MAGIC) {
        vec![load_from_memory(data)?]
    } else {
        tar::unpack(data)?
            .iter()
            .map(|file| load_from_memory(&file.content))
            .collect::<Result<_, _>>()?
    };

    Ok(images)
}

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error(transparent)]
    SplitIntoFragments(#[from] SplitIntoFragmentsError),
    #[error(transparent)]
    Encode(#[from] QrError),
    #[error(transparent)]
    EncodePngFiles(#[from] EncodePngFilesError),
    #[error(transparent)]
    PackTar(#[from] tar::PackError),
}

pub(crate) fn serialize(envelope: &envelope::Envelope) -> Result<Vec<u8>, SerializeError> {
    let fragments = split_into_fragments(envelope)?;
    let images = fragments
        .header
        .iter()
        .chain(fragments.ciphertext.iter())
        .map(|fragment| encode_image(&fragment.to_bytes()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tar::pack(&encode_png_files(&images)?)?)
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error(transparent)]
    FragmentDeserialize(#[from] FragmentDeserializeError),
    #[error(transparent)]
    ExtractImages(#[from] ExtractImagesError),
    #[error(transparent)]
    AssembleFragments(#[from] AssembleFragmentsError),
    #[error(transparent)]
    BinaryDeserialize(#[from] envelope::binary::DeserializeError),
}

pub(crate) fn deserialize_fragments(
    payloads: &[Vec<u8>],
) -> Result<Vec<Fragment>, FragmentDeserializeError> {
    payloads
        .iter()
        .map(|payload| Fragment::from_bytes(payload))
        .collect()
}

pub(crate) fn deserialize(data: &[u8]) -> Result<envelope::Envelope, DeserializeError> {
    let fragments = deserialize_fragments(&decode_images(&extract_images(data)?))?;
    let binary_envelope = assemble_fragments(fragments)?;
    let envelope = envelope::binary::deserialize(&binary_envelope)?;
    Ok(envelope)
}
