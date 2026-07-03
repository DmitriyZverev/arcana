use crate::envelope;
use crate::envelope::text::Encoding;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("Decoding error: {0}")]
    Decode(#[from] envelope::text::DecodeError),
    #[error(transparent)]
    Deserialize(#[from] serde_yaml::Error),
}

pub(crate) fn serialize(
    envelope: envelope::Envelope,
    encoding: Encoding,
) -> serde_yaml::Result<Vec<u8>> {
    let text_envelope = envelope::text::Envelope::encode(envelope, encoding);
    Ok(serde_yaml::to_string(&text_envelope)?.into_bytes())
}

pub(crate) fn deserialize(data: &[u8]) -> Result<envelope::Envelope, DeserializeError> {
    Ok(serde_yaml::from_slice::<envelope::text::Envelope>(data)?.try_into()?)
}
