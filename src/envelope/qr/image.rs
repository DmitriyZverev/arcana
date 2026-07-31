use image::{DynamicImage, GrayImage, Luma};
use qrcode::bits::Bits;
use qrcode::types::QrError;
use qrcode::{Color, EcLevel, QrCode, Version};

pub(super) const QR_CAPACITY: usize = 213;
const QR_VERSION: Version = Version::Normal(10);
const QR_EC_LEVEL: EcLevel = EcLevel::M;
const QR_PIXELS_PER_MODULE: u32 = 10;
const QR_QUIET_ZONE: u32 = 4;

pub(crate) fn encode_image(payload: &[u8]) -> Result<DynamicImage, QrError> {
    let mut bits = Bits::new(QR_VERSION);
    bits.push_byte_data(payload)?;
    bits.push_terminator(QR_EC_LEVEL)?;
    let code = QrCode::with_bits(bits, QR_EC_LEVEL)?;
    let module_count = code.width() as u32;
    let image_size = (module_count + QR_QUIET_ZONE * 2) * QR_PIXELS_PER_MODULE;
    let mut img = GrayImage::new(image_size, image_size);
    for pixel in img.pixels_mut() {
        *pixel = Luma([255u8]);
    }
    for (y, row) in code.to_colors().chunks(module_count as usize).enumerate() {
        for (x, &color) in row.iter().enumerate() {
            let px = (x as u32 + QR_QUIET_ZONE) * QR_PIXELS_PER_MODULE;
            let py = (y as u32 + QR_QUIET_ZONE) * QR_PIXELS_PER_MODULE;
            if color == Color::Dark {
                for dy in 0..QR_PIXELS_PER_MODULE {
                    for dx in 0..QR_PIXELS_PER_MODULE {
                        img.put_pixel(px + dx, py + dy, Luma([0u8]));
                    }
                }
            }
        }
    }
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
