# Arkana

_A modern CLI tool for password-based encryption with human-readable output._

## Status

> [!IMPORTANT]
> This project is in early development, and the API may change at any time. Not
> recommended for production use.

## Features

- **Strong Encryption**: `ChaCha20-Poly1305` authenticated encryption
- **Secure Key Derivation**: `Argon2id` with configurable parameters
- **Flexible I/O**: Stdin/stdout by default, with optional `--input-file` /
  `--output-file` flags
- **Configurable Encryption Parameters**: Override KDF and cipher settings
  per-invocation via CLI flags
- **Multiple Output Formats**: Human-readable YAML (default), compact binary, or
  QR code images via `--format`

## Usage

### Encrypt Data

```bash
# Read from stdin, write to stdout
echo "secret message" | arkana encrypt > encrypted.yml

# Read from file, write to file
arkana encrypt --input-file secret.txt --output-file encrypted.yml
```

### Decrypt Data

```bash
# Read from stdin, write to stdout
arkana decrypt < encrypted.yml > decrypted.txt

# Read from file, write to file
arkana decrypt --input-file encrypted.yml --output-file decrypted.txt
```

> [!NOTE]
> When `--input-file` is provided, stdin is ignored. When `--output-file` is
> provided, nothing is written to stdout.

### Encryption Parameters

Use `--kdf-*` and `--cipher-type` flags to override encryption parameters. When
omitted, the following defaults are used:

- **Argon2id**: memory 128 MiB, 4 iterations, parallelism 4
- **ChaCha20-Poly1305**

```bash
# Encrypt with a custom Argon2 algorithm
arkana encrypt --kdf-argon2-algorithm argon2i --input-file decrypted.txt

# Encrypt with reduced memory usage for faster key derivation
arkana encrypt --kdf-argon2-memory 65536 --input-file decrypted.txt

# Encrypt with all parameters explicitly specified
arkana encrypt \
  --kdf-type argon2 \
  --kdf-argon2-algorithm argon2id \
  --kdf-argon2-version 19 \
  --kdf-argon2-memory 131072 \
  --kdf-argon2-iterations 4 \
  --kdf-argon2-parallelism 4 \
  --cipher-type ChaCha20Poly1305 \
  --input-file decrypted.txt
```

> [!NOTE]
> Encryption parameters are stored in the container and are used automatically
> during decryption — no flags are needed on `arkana decrypt`.

### Encoding

Use `--format-yaml-encoding` to choose how binary values (`salt`, `nonce`,
`tag`, `ciphertext`) are represented in the YAML envelope. Supported values:
`base16`, `base32`, `base64` (default).

```bash
# Encrypt with base16 encoding
arkana encrypt --format-yaml-encoding base16 --input-file secret.txt --output-file encrypted.yml

# Encrypt with base32 encoding
arkana encrypt --format-yaml-encoding base32 --input-file secret.txt --output-file encrypted.yml
```

During decryption the encoding is read from the envelope — no flag is needed.

The `--format-yaml-encoding` flag is also available in `arkana convert` when
converting to YAML format:

```bash
# Convert binary to YAML with base16 encoding
arkana convert --from-format binary --to-format yaml --to-format-yaml-encoding base16 < encrypted.bin > encrypted.yml
```

> [!NOTE]
> The `--to-format-yaml-encoding` flag has no effect when `--format binary` is
> used — binary format stores raw bytes.

### Output Format

Use `--format` to select the envelope format. The default is `yaml`.

```bash
# Encrypt to binary format
arkana encrypt --format binary --input-file secret.txt --output-file encrypted.bin

# Decrypt from binary format
arkana decrypt --format binary --input-file encrypted.bin --output-file decrypted.txt

# Encrypt to QR code format (TAR archive of PNG images)
arkana encrypt --format qr --input-file secret.txt --output-file qr_codes.tar

# Decrypt from QR code format (TAR archive with images, PNG, or JPEG)
arkana decrypt --format qr --input-file qr_codes.tar --output-file decrypted.txt
arkana decrypt --format qr < qr_code.png > decrypted.txt
arkana decrypt --format qr < qr_code.jpeg > decrypted.txt
```

### Convert Between Formats

Use `arkana convert` to transform an encrypted envelope from one format to
another without decryption. No password is required.

```bash
# YAML to binary
arkana convert --from-format yaml --to-format binary < encrypted.yml > encrypted.bin

# Binary to YAML
arkana convert --from-format binary --to-format yaml --input-file encrypted.bin --output-file encrypted.yml

# YAML to QR
arkana convert --from-format yaml --to-format qr < encrypted.yml > qr_codes.tar

# QR to YAML
arkana convert --from-format qr --to-format yaml < qr_codes.tar > encrypted.yml
```

### Override Working Directory

Use the `--cwd` global flag to set the working directory for resolving all
relative file paths:

```bash
# Without --cwd, relative paths are resolved against the current working directory
cd /path/to/dir && arkana encrypt --input-file secret.txt --output-file encrypted.yml

# With --cwd, relative paths are resolved against the specified directory
arkana --cwd /path/to/dir encrypt --input-file secret.txt --output-file encrypted.yml
```

## Envelope Format

The encrypted data is stored in a self-describing envelope that contains all
parameters needed for decryption. Three formats are supported: YAML (default,
human-readable), binary (compact), and QR code (PNG images in a TAR archive).

### YAML

The default format is a human-readable YAML document:

```yaml
encoding: base64
params:
  kdf:
    type: argon2
    algorithm: argon2id
    version: 19
    memory: 131072
    iterations: 4
    parallelism: 4
    salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
  cipher:
    type: ChaCha20Poly1305
    nonce: CgoKCgoKCgoKCgoK
    tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
ciphertext: RmuSIEhbLyex+iTU
```

The `encoding` field specifies how binary values are encoded.

### Binary

The binary format is a compact, machine-readable encoding with the following
layout:

```
[ magic: 6B ][ version: 1B ][ params_len: 4B ][ ciphertext_len: 4B ][ params: CBOR ][ ciphertext ]
```

| Field            | Size                   | Description                                                                       |
| ---------------- | ---------------------- | --------------------------------------------------------------------------------- |
| `magic`          | 6 bytes                | Format identifier, always `0x61 0x72 0x63 0x61 0x6E 0x61` (ASCII string `arkana`) |
| `version`        | 1 byte                 | Format version; only `0x01` (`1`) is supported                                    |
| `params_len`     | 4 bytes (BE u32)       | Length of the `params` section in bytes                                           |
| `ciphertext_len` | 4 bytes (BE u32)       | Length of the `ciphertext` section in bytes                                       |
| `params`         | `params_len` bytes     | [CBOR](https://cbor.io/)-encoded encryption parameters                            |
| `ciphertext`     | `ciphertext_len` bytes | Raw encrypted bytes                                                               |

The `params` CBOR document contains the same fields as the YAML `params`
section, but binary values are stored as raw bytes rather than encoded strings.
The `ciphertext` section is likewise stored as raw bytes. The fixed-size header
(`magic` + `version` + `params_len` + `ciphertext_len`) makes the envelope
self-delimiting — the exact byte boundaries of every section are known after
reading the first 15 bytes. The maximum supported ciphertext size is 4 294 967
295 bytes (~4 GiB).

### QR

The QR format encodes a [binary-format](#binary) envelope into QR code images,
packaged as a TAR archive of PNG files. Useful for physical backups and paper
storage.

When the encrypted data exceeds the capacity of a single QR code, it is split
across multiple fragments. Each QR code is a self-contained symbol that includes
a format version, fragment index, total count, and a checksum for integrity
verification after reassembly.

Each fragment has the following layout:

```
[ version: 1B ][ index: 2B ][ total: 2B ][ checksum: 32B ][ chunk ]
```

| Field      | Size             | Description                                                                   |
| ---------- | ---------------- | ----------------------------------------------------------------------------- |
| `version`  | 1 byte           | Fragment format version; only `0x01` (`1`) is supported                       |
| `index`    | 2 bytes (BE u16) | 1-based position of this fragment among `total`                               |
| `total`    | 2 bytes (BE u16) | Total number of fragments the envelope was split into                         |
| `checksum` | 32 bytes         | SHA-256 checksum of the full reassembled data, identical across all fragments |
| `chunk`    | remaining bytes  | A chunk of the binary-format header or ciphertext, up to 176 bytes            |

The [binary-format](#binary) header and ciphertext are chunked into fragments
independently — the header's fragments always precede the ciphertext's
fragments, but each is split on its own, so a fragment never contains bytes from
both. Reassembling all fragments in order yields a full binary-format envelope.

Decryption accepts a TAR archive, a single PNG, or a single JPEG image — the
input type is auto-detected. A single image may contain multiple QR codes (e.g.,
a photo of a printed page).

## See also

- [ROADMAP.md](ROADMAP.md) — development plan and milestones
- [PRINCIPLES.md](PRINCIPLES.md) — design principles
- [CONTRIBUTING.md](CONTRIBUTING.md) — development setup and workflow

## License

This project is licensed under the [MIT License](LICENSE).
