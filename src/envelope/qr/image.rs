use image::{DynamicImage, Luma};
use qrcode::bits::Bits;
use qrcode::types::QrError;
use qrcode::{EcLevel, QrCode, Version};

pub(super) const QR_CAPACITY: usize = 213;
const QR_VERSION: Version = Version::Normal(10);
const QR_EC_LEVEL: EcLevel = EcLevel::M;
const QR_PIXELS_PER_MODULE: u32 = 10;

pub(crate) fn encode_image(payload: &[u8]) -> Result<DynamicImage, QrError> {
    let mut bits = Bits::new(QR_VERSION);
    bits.push_byte_data(payload)?;
    bits.push_terminator(QR_EC_LEVEL)?;
    let code = QrCode::with_bits(bits, QR_EC_LEVEL)?;
    let img = code
        .render::<Luma<u8>>()
        .module_dimensions(QR_PIXELS_PER_MODULE, QR_PIXELS_PER_MODULE)
        .build();
    Ok(DynamicImage::ImageLuma8(img))
}

#[derive(Debug, thiserror::Error)]
enum DecodeQrError {
    #[error(transparent)]
    Detect(#[from] rxing::Exceptions),
    #[error("No QR code found in image")]
    NotFound,
}

fn decode_image(image: &DynamicImage) -> Result<Vec<Vec<u8>>, DecodeQrError> {
    let gray = image.to_luma8();
    let (w, h) = gray.dimensions();
    let luma = gray.into_raw();
    let results = rxing::helpers::detect_multiple_in_luma(luma, w, h)?;
    if results.is_empty() {
        return Err(DecodeQrError::NotFound);
    }
    Ok(results
        .iter()
        .map(|result| result.getRawBytes().to_vec())
        .collect())
}

pub(crate) fn decode_images(images: &[DynamicImage]) -> Vec<Vec<u8>> {
    let mut payloads = Vec::new();
    for image in images {
        match decode_image(image) {
            Ok(decoded) => payloads.extend(decoded),
            Err(DecodeQrError::NotFound | DecodeQrError::Detect(_)) => continue,
        }
    }
    payloads
}
