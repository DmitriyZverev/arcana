use crate::envelope::{self, Envelope};
use thiserror::Error;

pub enum OutputFormat {
    Yaml(envelope::text::yaml::SerializeParams),
    Binary,
    Qr,
}

pub enum InputFormat {
    Yaml,
    Binary,
    Qr,
}

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Binary(#[from] envelope::binary::SerializeError),
    #[error(transparent)]
    Qr(#[from] envelope::qr::SerializeError),
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error(transparent)]
    Yaml(#[from] envelope::text::yaml::DeserializeError),
    #[error(transparent)]
    Binary(#[from] envelope::binary::DeserializeError),
    #[error(transparent)]
    Qr(#[from] envelope::qr::DeserializeError),
}

pub(crate) fn serialize(
    envelope: Envelope,
    format: OutputFormat,
) -> Result<Vec<u8>, SerializeError> {
    match format {
        OutputFormat::Yaml(params) => Ok(envelope::text::yaml::serialize(envelope, &params)?),
        OutputFormat::Binary => Ok(envelope::binary::serialize(&envelope)?),
        OutputFormat::Qr => Ok(envelope::qr::serialize(&envelope)?),
    }
}

pub(crate) fn deserialize(data: &[u8], format: InputFormat) -> Result<Envelope, DeserializeError> {
    match format {
        InputFormat::Yaml => Ok(envelope::text::yaml::deserialize(data)?),
        InputFormat::Binary => Ok(envelope::binary::deserialize(data)?),
        InputFormat::Qr => Ok(envelope::qr::deserialize(data)?),
    }
}
