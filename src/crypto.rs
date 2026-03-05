#[cfg(not(feature = "deterministic"))]
use argon2::password_hash::rand_core::{OsRng, RngCore};
use argon2::{Algorithm, Argon2, Params, Version};
#[cfg(not(feature = "deterministic"))]
use chacha20poly1305::AeadCore;
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroizing;

mod base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = STANDARD.encode(bytes);
        let wrapped = encoded
            .as_bytes()
            .chunks(64)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        serializer.serialize_str(&wrapped)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let cleaned: String = s.lines().collect();
        STANDARD.decode(cleaned).map_err(serde::de::Error::custom)
    }
}

mod hex_serde {
    use serde::{Deserialize, Deserializer};

    pub fn serialize<S, const N: usize>(value: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = hex::encode_upper(value);
        serializer.serialize_str(&hex)
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != N {
            return Err(serde::de::Error::custom("invalid length"));
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

mod version_serde {
    use argon2::Version;
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let num = match value {
            Version::V0x10 => 0x10u32,
            Version::V0x13 => 0x13u32,
        };
        serializer.serialize_u32(num)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = u32::deserialize(deserializer)?;
        match num {
            0x10 => Ok(Version::V0x10),
            0x13 => Ok(Version::V0x13),
            _ => Err(D::Error::custom("unsupported Argon2 version")),
        }
    }
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Hashing error")]
    Hashing,
}

#[derive(Error, Debug)]
pub enum EncryptError {
    #[error("Hashing error")]
    Hashing,
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

#[derive(Error, Debug)]
pub enum DecryptError {
    #[error("Decryption error")]
    Decryption,
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

#[derive(Default)]
pub struct EncryptParams {
    pub data: Vec<u8>,
    pub password: Zeroizing<Vec<u8>>,
    pub kdf: Kdf,
    pub cipher: Cipher,
}

impl EncryptParams {
    pub fn new(data: Vec<u8>, password: Zeroizing<Vec<u8>>) -> Self {
        Self {
            data,
            password,
            ..Default::default()
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Cipher {
    #[default]
    ChaCha20Poly1305,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Argon2Facade {
    #[serde(with = "version_serde")]
    pub version: Version,
    pub memory: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

impl Default for Argon2Facade {
    fn default() -> Self {
        Self {
            version: Version::default(),
            memory: 128 * 1024,
            iterations: 4,
            parallelism: 4,
        }
    }
}

impl Argon2Facade {
    fn hash_password(
        &self,
        salt: &[u8],
        password: &[u8],
    ) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
        let mut key = Zeroizing::new([0u8; KEY_LEN]);
        Argon2::new(
            Algorithm::Argon2id,
            self.version,
            Params::new(
                self.memory,
                self.iterations,
                self.parallelism,
                Some(KEY_LEN),
            )
            .map_err(|_| CryptoError::Hashing)?,
        )
        .hash_password_into(password, salt, key.as_mut())
        .map_err(|_| CryptoError::Hashing)?;
        Ok(key)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Kdf {
    Argon2id(Argon2Facade),
}

impl Default for Kdf {
    fn default() -> Self {
        Self::Argon2id(Argon2Facade::default())
    }
}

#[derive(Deserialize, Serialize)]
pub struct EncryptedContainer {
    pub kdf: Kdf,
    pub cipher: Cipher,
    #[serde(with = "hex_serde")]
    pub salt: [u8; SALT_LEN],
    #[serde(with = "hex_serde")]
    pub nonce: [u8; NONCE_LEN],
    #[serde(with = "base64_serde")]
    pub ciphertext: Vec<u8>,
}

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[cfg(feature = "deterministic")]
fn generate_salt() -> Result<[u8; SALT_LEN], EncryptError> {
    Ok([0x1B; SALT_LEN])
}

#[cfg(not(feature = "deterministic"))]
fn generate_salt() -> Result<[u8; SALT_LEN], EncryptError> {
    let mut salt = [0u8; SALT_LEN];
    OsRng
        .try_fill_bytes(&mut salt)
        .map_err(|_| EncryptError::Hashing)?;
    Ok(salt)
}

#[cfg(feature = "deterministic")]
fn generate_nonce() -> [u8; NONCE_LEN] {
    [0x0A; NONCE_LEN]
}

#[cfg(not(feature = "deterministic"))]
fn generate_nonce() -> [u8; NONCE_LEN] {
    ChaCha20Poly1305::generate_nonce(OsRng).into()
}

pub fn encrypt(encrypt_params: EncryptParams) -> Result<EncryptedContainer, EncryptError> {
    let salt = generate_salt()?;
    let key = match &encrypt_params.kdf {
        Kdf::Argon2id(argon2) => argon2.hash_password(&salt, &encrypt_params.password)?,
    };

    let (nonce, ciphertext) = match &encrypt_params.cipher {
        Cipher::ChaCha20Poly1305 => {
            let nonce = generate_nonce();
            (
                nonce,
                ChaCha20Poly1305::new(Key::from_slice(key.as_ref()))
                    .encrypt(Nonce::from_slice(&nonce), encrypt_params.data.as_slice())
                    .map_err(|_| EncryptError::Hashing)?,
            )
        }
    };

    Ok(EncryptedContainer {
        salt,
        nonce,
        ciphertext,
        cipher: encrypt_params.cipher,
        kdf: encrypt_params.kdf,
    })
}

pub fn decrypt(
    encrypted_container: &EncryptedContainer,
    password: &[u8],
) -> Result<Vec<u8>, DecryptError> {
    let key = match &encrypted_container.kdf {
        Kdf::Argon2id(argon2) => argon2.hash_password(&encrypted_container.salt, password)?,
    };
    let ciphertext = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()))
        .decrypt(
            Nonce::from_slice(&encrypted_container.nonce),
            encrypted_container.ciphertext.as_slice(),
        )
        .map_err(|_| DecryptError::Decryption)?;

    Ok(ciphertext)
}
