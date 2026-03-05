# Arcana

_A modern CLI tool for password-based encryption with human-readable output._

## Features

- **Strong Encryption**: `ChaCha20-Poly1305` authenticated encryption
- **Secure Key Derivation**: `Argon2id` with configurable parameters
- **Stdin/Stdout Interface**: Unix-friendly pipeline integration
- **YAML Output Format**: Human-readable encrypted containers

## Usage

### Encrypt Data

```bash
# Read password interactively
echo "secret message" | arcana encrypt > encrypted.yml

# Use password from file
echo "secret message" | arcana encrypt --password-file password.txt > encrypted.yml
```

### Decrypt Data

```bash
# Read password interactively
arcana decrypt < encrypted.yml > decrypted.txt

# Use password from file
arcana decrypt --password-file password.txt < encrypted.yml > decrypted.txt
```

## Encrypted Container Format

The encrypted data is stored in a human-readable YAML format that describes all necessary settings for decryption:

```yaml
kdf:
  type: Argon2id
  version: 19
  memory: 131072
  iterations: 4
  parallelism: 4
cipher:
  type: ChaCha20Poly1305
salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
ciphertext: |-
  QmGMKUpbMTiw4y3VNcyRczjoWe3tDsafehFLdnVsACiAdBg4AH91oORjkONGmEN+
  QpRkityFXVFY/FdiCmC6+0xo5TZwuhY55fKfiVw1oVUUbQvUu54uiZWc8iibZ+H9
  80N4XRKNKiFvUA7DbG3rMO+RomI4hyGM0l5S3E5LZEALkoV6ivpWeKHyOsCuef+J
  LmFJ

```

## Security Parameters

- **Argon2id**: Memory-hard key derivation (128 MiB, 4 iterations, parallelism 4)
- **ChaCha20-Poly1305**: Authenticated encryption with 256-bit keys
- **Salt**: 256-bit random salt per encryption
- **Nonce**: 96-bit random nonce per encryption
