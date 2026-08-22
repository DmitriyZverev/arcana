use image::{DynamicImage, Luma};
use qrcode::bits::Bits;
use qrcode::types::QrError;
use qrcode::{EcLevel, QrCode, Version};
use rxing::common::{DetectorRXingResult, HybridBinarizer};
use rxing::multi::qrcode::detector::MultiDetector;
use rxing::qrcode::decoder::qrcode_decoder::decode_bitmatrix_with_hints;
use rxing::{BinaryBitmap, DecodeHintValue, DecodeHints, Luma8LuminanceSource};

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
    let image = image.to_luma8();
    let (width, height) = image.dimensions();
    let hints = DecodeHints::default().with(DecodeHintValue::TryHarder(true));
    let source = Luma8LuminanceSource::new(image.into_raw(), width, height);
    let bitmap = BinaryBitmap::new(HybridBinarizer::new(source));
    let detected = MultiDetector::new(bitmap.get_black_matrix()).detectMulti(&hints)?;
    let payloads: Vec<Vec<u8>> = detected
        .iter()
        .filter_map(|result| {
            let decoded = decode_bitmatrix_with_hints(result.getBits(), &hints).ok()?;
            Some(decoded.getByteSegments().concat())
        })
        .collect();
    if payloads.is_empty() {
        return Err(DecodeQrError::NotFound);
    }
    Ok(payloads)
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
