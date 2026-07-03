pub(crate) mod yaml;

use crate::envelope;
use crate::envelope::Argon2Params;
use data_encoding::{BASE32, BASE64, HEXUPPER, HEXUPPER_PERMISSIVE};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum DecodeError {
    #[error("Invalid length: expected {expected}, actual {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error(transparent)]
    Decode(#[from] data_encoding::DecodeError),
}

fn encode_bytes(bytes: &[u8], encoding: &Encoding) -> String {
    let encoded = match encoding {
        Encoding::Base16 => HEXUPPER.encode(bytes),
        Encoding::Base32 => BASE32.encode(bytes),
        Encoding::Base64 => BASE64.encode(bytes),
    };
    encoded
        .as_bytes()
        .chunks(64)
        .map(|chunk| std::str::from_utf8(chunk).expect("Encoding is ASCII"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_bytes(s: &str, encoding: &Encoding) -> Result<Vec<u8>, data_encoding::DecodeError> {
    let cleaned: String = s.lines().collect();
    match encoding {
        Encoding::Base16 => HEXUPPER_PERMISSIVE.decode(cleaned.as_bytes()),
        Encoding::Base32 => BASE32.decode(cleaned.as_bytes()),
        Encoding::Base64 => BASE64.decode(cleaned.as_bytes()),
    }
}

fn decode_fixed<const N: usize>(s: &str, encoding: &Encoding) -> Result<[u8; N], DecodeError> {
    let bytes = decode_bytes(s, encoding)?;
    let len = bytes.len();
    bytes.try_into().map_err(|_| DecodeError::InvalidLength {
        expected: N,
        actual: len,
    })
}

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub enum Encoding {
    #[serde(rename = "base16")]
    Base16,
    #[serde(rename = "base32")]
    Base32,
    #[default]
    #[serde(rename = "base64")]
    Base64,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum CipherParams {
    #[serde(rename = "ChaCha20Poly1305")]
    ChaCha20Poly1305 { nonce: String, tag: String },
}

impl CipherParams {
    fn encode(cipher: envelope::CipherParams, encoding: &Encoding) -> Self {
        match cipher {
            envelope::CipherParams::ChaCha20Poly1305 { nonce, tag } => {
                CipherParams::ChaCha20Poly1305 {
                    nonce: encode_bytes(&nonce, encoding),
                    tag: encode_bytes(&tag, encoding),
                }
            }
        }
    }
}

impl envelope::CipherParams {
    fn decode(cipher: CipherParams, encoding: &Encoding) -> Result<Self, DecodeError> {
        Ok(match cipher {
            CipherParams::ChaCha20Poly1305 { nonce, tag } => {
                envelope::CipherParams::ChaCha20Poly1305 {
                    nonce: decode_fixed(&nonce, encoding)?,
                    tag: decode_fixed(&tag, encoding)?,
                }
            }
        })
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub(crate) enum KdfParams {
    #[serde(rename = "argon2")]
    Argon2 {
        #[serde(flatten)]
        params: Argon2Params,
        salt: String,
    },
}

impl KdfParams {
    fn encode(kdf: envelope::KdfParams, encoding: &Encoding) -> Self {
        match kdf {
            envelope::KdfParams::Argon2 { params, salt } => KdfParams::Argon2 {
                params,
                salt: encode_bytes(&salt, encoding),
            },
        }
    }
}

impl envelope::KdfParams {
    fn decode(kdf: KdfParams, encoding: &Encoding) -> Result<Self, DecodeError> {
        Ok(match kdf {
            KdfParams::Argon2 { params, salt } => envelope::KdfParams::Argon2 {
                params,
                salt: decode_fixed(&salt, encoding)?,
            },
        })
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EnvelopeParams {
    pub kdf: KdfParams,
    pub cipher: CipherParams,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Envelope {
    pub encoding: Encoding,
    pub params: EnvelopeParams,
    pub ciphertext: String,
}

impl Envelope {
    pub(crate) fn encode(envelope: envelope::Envelope, encoding: Encoding) -> Self {
        Envelope {
            encoding,
            params: EnvelopeParams {
                kdf: KdfParams::encode(envelope.params.kdf, &encoding),
                cipher: CipherParams::encode(envelope.params.cipher, &encoding),
            },
            ciphertext: encode_bytes(&envelope.ciphertext, &encoding),
        }
    }
}

impl TryFrom<Envelope> for envelope::Envelope {
    type Error = DecodeError;

    fn try_from(envelope: Envelope) -> Result<Self, DecodeError> {
        Ok(envelope::Envelope {
            params: envelope::EnvelopeParams {
                kdf: envelope::KdfParams::decode(envelope.params.kdf, &envelope.encoding)?,
                cipher: envelope::CipherParams::decode(envelope.params.cipher, &envelope.encoding)?,
            },
            ciphertext: decode_bytes(&envelope.ciphertext, &envelope.encoding)?,
        })
    }
}
