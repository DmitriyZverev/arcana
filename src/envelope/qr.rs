use crate::envelope;
use image::{DynamicImage, ImageError};
use qrcode::types::QrError;
use qrcode::{EcLevel, Version};
use sha2::{Digest, Sha256};
use std::io::Read;

type FormatVersion = u8;
type FragmentIndex = u16;
type FragmentTotal = u16;
type Sha256Type = [u8; SHA_256_SIZE];

const QR_VERSION: Version = Version::Normal(10);
const QR_EC_LEVEL: EcLevel = EcLevel::M;
const QR_CAPACITY: usize = 213;
const QR_PIXELS_PER_MODULE: u32 = 10;
const QR_QUIET_ZONE: u32 = 4;
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

struct Fragment {
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
    let total_fragments = header_bytes.len().div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE)
        + ciphertext_bytes
            .len()
            .div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE)
            .max(1);
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
    let header_count = header_bytes.len().div_ceil(MAX_FRAGMENT_PAYLOAD_SIZE);
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

fn encode_qr(fragment: &Fragment) -> Result<DynamicImage, QrError> {
    use qrcode::QrCode;
    use qrcode::bits::Bits;
    let payload = fragment.to_bytes();
    let mut bits = Bits::new(QR_VERSION);
    bits.push_byte_data(&payload)?;
    bits.push_terminator(QR_EC_LEVEL)?;
    let code = QrCode::with_bits(bits, QR_EC_LEVEL)?;
    let module_count = code.width() as u32;
    let image_size = (module_count + QR_QUIET_ZONE * 2) * QR_PIXELS_PER_MODULE;
    let mut img = image::GrayImage::new(image_size, image_size);
    for pixel in img.pixels_mut() {
        *pixel = image::Luma([255u8]);
    }
    for (y, row) in code.to_colors().chunks(module_count as usize).enumerate() {
        for (x, &color) in row.iter().enumerate() {
            let px = (x as u32 + QR_QUIET_ZONE) * QR_PIXELS_PER_MODULE;
            let py = (y as u32 + QR_QUIET_ZONE) * QR_PIXELS_PER_MODULE;
            if color == qrcode::Color::Dark {
                for dy in 0..QR_PIXELS_PER_MODULE {
                    for dx in 0..QR_PIXELS_PER_MODULE {
                        img.put_pixel(px + dx, py + dy, image::Luma([0u8]));
                    }
                }
            }
        }
    }
    Ok(DynamicImage::ImageLuma8(img))
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeQrError {
    #[error(transparent)]
    Detect(#[from] rxing::Exceptions),
    #[error("No QR code found in image")]
    NotFound,
    #[error(transparent)]
    FragmentDeserialize(#[from] FragmentDeserializeError),
}

fn decode_qr(image: &DynamicImage) -> Result<Vec<Fragment>, DecodeQrError> {
    let gray = image.to_luma8();
    let (w, h) = gray.dimensions();
    let luma = gray.into_raw();
    let results = rxing::helpers::detect_multiple_in_luma(luma, w, h)?;
    if results.is_empty() {
        return Err(DecodeQrError::NotFound);
    }
    let mut fragments = Vec::with_capacity(results.len());
    for result in &results {
        let bytes = result.getRawBytes();
        fragments.push(Fragment::from_bytes(bytes)?);
    }
    Ok(fragments)
}

#[derive(Debug, thiserror::Error)]
pub enum PackTarError {
    #[error(transparent)]
    CreateImage(#[from] ImageError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn pack_tar(images: Vec<DynamicImage>) -> Result<Vec<u8>, PackTarError> {
    let mut archive = tar::Builder::new(Vec::new());
    for (i, image) in images.iter().enumerate() {
        let mut png_data = Vec::new();
        image.write_to(
            &mut std::io::Cursor::new(&mut png_data),
            image::ImageFormat::Png,
        )?;
        let name = format!("{:05}.png", i + 1);
        let mut header = tar::Header::new_gnu();
        header.set_size(png_data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append_data(&mut header, &name, png_data.as_slice())?;
    }
    archive.finish()?;
    Ok(archive.into_inner()?)
}

#[derive(Debug, thiserror::Error)]
pub enum UnpackTarError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn unpack_tar(bytes: &[u8]) -> Result<Vec<Vec<u8>>, UnpackTarError> {
    let mut archive = tar::Archive::new(bytes);
    let mut entries_bytes = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let mut entry_bytes = Vec::new();
        entry.read_to_end(&mut entry_bytes)?;
        entries_bytes.push(entry_bytes);
    }
    Ok(entries_bytes)
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeFragmentsError {
    #[error(transparent)]
    LoadImage(#[from] ImageError),
    #[error(transparent)]
    UnpackTar(#[from] UnpackTarError),
    #[error(transparent)]
    DecodeQr(#[from] DecodeQrError),
}

fn decode_fragments(data: &[u8]) -> Result<Vec<Fragment>, DecodeFragmentsError> {
    let images: Vec<DynamicImage> = if data.starts_with(PNG_MAGIC) || data.starts_with(JPEG_MAGIC) {
        vec![image::load_from_memory(data)?]
    } else {
        unpack_tar(data)?
            .iter()
            .map(|bytes| image::load_from_memory(bytes))
            .collect::<Result<_, _>>()?
    };
    let mut fragments = Vec::new();
    for image in &images {
        match decode_qr(image) {
            Ok(frags) => fragments.extend(frags),
            Err(DecodeQrError::NotFound | DecodeQrError::Detect(_)) => {}
            Err(e @ DecodeQrError::FragmentDeserialize(_)) => {
                return Err(DecodeFragmentsError::DecodeQr(e));
            }
        }
    }
    Ok(fragments)
}

#[derive(Debug, thiserror::Error)]
pub enum SerializeError {
    #[error(transparent)]
    SplitIntoFragments(#[from] SplitIntoFragmentsError),
    #[error(transparent)]
    Encode(#[from] QrError),
    #[error(transparent)]
    PackTar(#[from] PackTarError),
}

pub(crate) fn serialize(envelope: &envelope::Envelope) -> Result<Vec<u8>, SerializeError> {
    let fragments = split_into_fragments(envelope)?;
    let images = fragments
        .header
        .iter()
        .chain(fragments.ciphertext.iter())
        .map(encode_qr)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(pack_tar(images)?)
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeError {
    #[error(transparent)]
    Decode(#[from] DecodeFragmentsError),
    #[error(transparent)]
    AssembleFragments(#[from] AssembleFragmentsError),
    #[error(transparent)]
    BinaryDeserialize(#[from] envelope::binary::DeserializeError),
}

pub(crate) fn deserialize(data: &[u8]) -> Result<envelope::Envelope, DeserializeError> {
    let fragments = decode_fragments(data)?;
    let binary_data = assemble_fragments(fragments)?;
    let envelope = envelope::binary::deserialize(&binary_data)?;
    Ok(envelope)
}
