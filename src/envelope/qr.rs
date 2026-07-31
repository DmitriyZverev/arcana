pub(crate) mod fragment;
pub(crate) mod image;
pub(crate) mod tar;

use crate::envelope;
use ::image::{DynamicImage, ImageError, ImageFormat, load_from_memory};
use fragment::{
    AssembleFragmentsError, EnvelopeFragmentationError, FragmentDeserializeError,
    FragmentedEnvelope, assemble_fragments, deserialize_fragments,
};
use image::{decode_images, encode_image};
use qrcode::types::QrError;

const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8];

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
    EnvelopeFragmentation(#[from] EnvelopeFragmentationError),
    #[error(transparent)]
    Encode(#[from] QrError),
    #[error(transparent)]
    EncodePngFiles(#[from] EncodePngFilesError),
    #[error(transparent)]
    PackTar(#[from] tar::PackError),
}

pub(crate) fn serialize(envelope: &envelope::Envelope) -> Result<Vec<u8>, SerializeError> {
    let fragmented_envelope = FragmentedEnvelope::from_envelope(envelope)?;
    let images = fragmented_envelope
        .fragments()
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

pub(crate) fn deserialize(data: &[u8]) -> Result<envelope::Envelope, DeserializeError> {
    let fragments = deserialize_fragments(&decode_images(&extract_images(data)?))?;
    let binary_envelope = assemble_fragments(fragments)?;
    let envelope = envelope::binary::deserialize(&binary_envelope)?;
    Ok(envelope)
}
