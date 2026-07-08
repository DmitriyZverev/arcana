# Roadmap

_This document outlines the planned development stages for `arkana`._

_Each step builds on the previous one, progressively expanding the interface
from basic stdin/stdout piping to a full interactive experience_

## Status legend

| Badge         | Meaning                            |
| ------------- | ---------------------------------- |
| `Done`        | Fully implemented and available    |
| `In progress` | Work has started, not yet complete |
| `Planned`     | Scheduled, work not yet started    |

## Milestones

### Step 1 — Basic stdin/stdout interface `Done`

Encrypt and decrypt data using standard input/output streams. This is the
minimal viable interface and serves as the foundation for all later steps.

```shell
arkana encrypt < decrypted.txt > encrypted.yml
arkana decrypt < encrypted.yml > decrypted.txt
```

### Step 2 — File path arguments `Done`

Add `--input` and `--output` flags as an alternative to stream redirection.
Useful when integrating with scripts or tools that work with file paths
directly.

```shell
arkana encrypt --input ./decrypted.txt --output ./encrypted.yml
arkana decrypt --input ./encrypted.yml --output ./decrypted.txt
```

### Step 3 — Encryption parameters `Done`

Add flags to override KDF and cipher settings per-invocation.

```shell
arkana encrypt --kdf-type argon2 \
               --kdf-argon2-algorithm argon2id \
               --kdf-argon2-version 19 \
               --kdf-argon2-memory 65536 \
               --kdf-argon2-iterations 3 \
               --kdf-argon2-parallelism 1 \
               --cipher-type ChaCha20Poly1305 < decrypted.txt > encrypted.yml
```

Supported flags:

- `--kdf-type <type>` — select key derivation function (`argon2`)
- `--kdf-argon2-algorithm <algorithm>` — Argon2 algorithm (`argon2id`,
  `argon2i`, `argon2d`; default: `argon2id`)
- `--kdf-argon2-version <version>` — Argon2 version (`16`, `19`; default: `19`)
- `--kdf-argon2-memory <kib>` — memory in KiB (default: `131072`)
- `--kdf-argon2-iterations <n>` — number of iterations (default: `4`)
- `--kdf-argon2-parallelism <n>` — degree of parallelism (default: `4`)
- `--cipher-type <type>` — select cipher (`ChaCha20Poly1305`)

### Step 4 — Configuration file `Planned`

Add support for a configuration file at `$HOME/.arkana/config.toml` for setting
default encryption parameters:

```toml
# $HOME/.arkana/config.toml
[kdf]
type = "argon2"

[kdf.argon2]
algorithm = "argon2id"
memory = 65536
iterations = 3
parallelism = 1

[cipher]
type = "chacha20poly1305"
```

The default config path can be overridden with `--config`:

```shell
arkana --config /path/to/config.toml encrypt < decrypted.txt > encrypted.yml
```

CLI flags (Step 3) take precedence over config file values.

### Step 5 — Encoding field `Done`

Add an `encoding` field to the YAML envelope that controls how binary values
(`salt`, `nonce`, `tag`, `ciphertext`) are represented. Supported values:
`base16`, `base32`, `base64`.

```yaml
encoding: base16 | base32 | base64
kdf:
  type: argon2
  # ...
  salt: <encoded 256-bit salt>
cipher:
  type: ChaCha20Poly1305
  nonce: <encoded 96-bit nonce>
  tag: <encoded 128-bit authentication tag>
  ciphertext: <encoded ciphertext>
```

The default encoding is `base64` — existing behavior is unchanged.

A new `--encoding` flag is available during encryption:

```shell
arkana encrypt --encoding base16 < decrypted.txt > encrypted.yml
```

During decryption the `encoding` field is read from the envelope — no flag is
needed.

### Step 6 — Binary container format `Done`

Add `--format` flag to `encrypt` and `decrypt` commands with two supported
values: `yaml` (default) and `binary`. The binary format serializes the
encrypted container using CBOR (Concise Binary Object Representation) — a
compact, binary encoding of the same fields as the YAML container.

```shell
arkana encrypt --format binary < decrypted.txt > encrypted.bin
arkana decrypt --format binary < encrypted.bin > decrypted.txt
```

The default format is `yaml` — existing behavior is unchanged.

The binary format does not include the `encoding` field (Step 5) — all binary
values are stored as raw bytes in CBOR. The `--encoding` flag is ignored when
`--format binary` is used.

### Step 7 — Format conversion `Done`

Add a `convert` command that transforms an encrypted envelope from one format to
another without decryption. The envelope content is preserved exactly — no
password is required and no re-encryption occurs.

```shell
arkana convert --from-format yaml --to-format binary --input envelope.yml --output envelope.bin
arkana convert --from-format binary --to-format yaml --input envelope.bin --output envelope.yml
```

Both `--from-format` and `--to-format` are required. Supported values match the
`--format` flag: `yaml`, `binary`, and (after Step 8) `qr`.

Standard I/O is supported:

```shell
arkana convert --from-format yaml --to-format binary < envelope.yml > envelope.bin
```

### Step 8 — QR code format `Done`

Add `qr` as a new value for the `--format` flag, enabling QR code images as an
alternative container format. Useful for physical backups and paper storage.

```shell
arkana encrypt --format qr < decrypted.txt > qr_codes.tar

# tar archive can contain multiple related QR code images
arkana decrypt --format qr < qr_codes.tar > decrypted.txt
arkana decrypt --format qr --input qr_codes.tar --output decrypted.txt

# or a single QR code jpeg image
arkana decrypt --format qr < qr_code.jpeg > decrypted.txt
arkana decrypt --format qr --input qr_code.jpeg --output decrypted.txt

# or a single QR code png image
arkana decrypt --format qr < qr_code.png > decrypted.txt
arkana decrypt --format qr --input qr_code.png --output decrypted.txt
```

Encrypt always outputs a TAR archive containing one or more PNG images. When the
encrypted container exceeds the capacity of a single QR code, it is split across
multiple independent symbols, each readable by any standard QR scanner.

Decrypt accepts a TAR archive, a PNG, or a JPEG image — auto-detected from the
input. A single image may contain multiple QR codes (e.g., a photo of a printed
page). Images within a TAR archive need not be ordered.

Each QR code symbol encodes a binary payload in the following format:

```
[1 byte]  version  — format version (currently 0x01)
[2 bytes] index    — 1-based position of this symbol in the sequence (u16 big-endian)
[2 bytes] total    — total number of symbols in the sequence (u16 big-endian)
[32 bytes] sha256  — SHA-256 checksum of the complete encrypted container, identical
                     across all symbols; used to group symbols belonging to the same
                     container and to verify integrity after assembly
[N bytes] fragment — a binary fragment of the CBOR-encoded encrypted container
```

### Step 9 — PDF render (`arkana render`) `Planned`

Add a new `render` command that produces a printable PDF document from an
encrypted envelope. The PDF serves as a physical backup — it contains QR codes
(CBOR-encoded) and a formatted human-readable representation of the envelope
fields.

```shell
# From stdin (default format: yaml):
arkana encrypt | arkana render --output backup.pdf

# From file:
arkana render --input envelope.yaml --output backup.pdf

# Binary format:
arkana encrypt --format binary | arkana render --format binary --output backup.pdf

# To stdout:
arkana encrypt | arkana render > backup.pdf
```

`render` accepts an envelope in any supported format via `--format` (`yaml`,
`binary`, `qr`; default: `yaml`). The output is always a PDF file.

**PDF layout:**

The PDF consists of two sections, in order:

1. **QR code pages** — each page contains up to 6 QR codes arranged in a 2×3
   grid (2 columns, 3 rows). QR codes use version 10 and encode the same binary
   payload format as `--format qr` (Step 8). If there are more than 6 QR codes,
   they continue on later pages.

2. **Envelope detail pages** — a formatted, human-readable representation of the
   envelope fields (KDF parameters, cipher type, nonce, tag, truncated
   ciphertext) with graphical elements (lines, tables). This is not raw YAML —
   it is a custom-rendered representation.

**Page metadata (on every page):**

- Page number / total pages
- SHA-256 checksum of the CBOR-encoded envelope (same checksum as in QR payload)
- Timestamp of PDF generation

### Step 10 — Named secret storage `Planned`

Introduce a secret registry stored in `$HOME/.arkana/secrets/`. Each encryption
creates a new versioned snapshot of the secret, making it possible to track and
restore previous versions. Secrets are always stored in YAML format.

The secrets directory can be overridden via `--secrets-dir` or via
`config.toml`:

```shell
arkana --secrets-dir /path/to/secrets secret encrypt <secret-name> < ./decrypted.txt
```

```toml
# $HOME/.arkana/config.toml
[secrets]
dir = "/path/to/secrets"
```

`--secrets-dir` takes precedence over the config file value. If neither is set,
`$HOME/.arkana/secrets/` is used.

File naming pattern: `<secret-name>.YYYY_MM_DD_HH_mm_ss_fffffffff_<counter>.yml`

The timestamp in the filename is always in UTC. The latest version is determined
by this timestamp (and counter as a tiebreaker).

**Encrypting a named secret:**

```shell
# From stdin:
arkana secret encrypt <secret-name> < ./decrypted.txt

# From file:
arkana secret encrypt <secret-name> --input ./decrypted.txt
```

Both commands write the encrypted result to:
`$HOME/.arkana/secrets/<secret-name>.YYYY_MM_DD_HH_mm_ss_fffffffff_<counter>.yml`

Each invocation creates a new version; existing versions are never modified.
`--output` is not supported — the destination is always the secrets directory.

**Decrypting a named secret:**

```shell
# To stdout:
arkana secret decrypt <secret-name> > ./decrypted.txt

# To file:
arkana secret decrypt <secret-name> --output ./decrypted.txt
```

**Decrypting a specific version:**

```shell
# To stdout:
arkana secret decrypt <secret-name> --version 2024_03_16_130000_000000000_0001 > ./decrypted.txt

# To file:
arkana secret decrypt <secret-name> --version 2024_03_16_130000_000000000_0001 --output ./decrypted.txt
```

The version identifier matches the filename suffix returned by
`arkana secret list-versions <name>`. Without `--version`, the latest version is
used. Exits with an error if the secret or version does not exist.

**Listing all secrets:**

```shell
arkana secret list
```

Outputs the list of secret names stored in `$HOME/.arkana/secrets/`:

```
foo
bar
baz
```

**Listing versions of a secret:**

```shell
arkana secret list-versions <secret-name>
```

Outputs the list of available versions for the specified secret, ordered from
oldest to newest:

```
2024_03_16_120000_000000000_0001
2024_03_16_130000_000000000_0001
2024_03_17_090000_000000000_0001
```

**Deleting a secret or a specific version:**

```shell
# Delete all versions of a secret:
arkana secret delete <secret-name>

# Delete a specific version:
arkana secret delete <secret-name> --version 2024_03_16_120000_000000000_0001
```

Without `--version`, all versions are deleted. Any deletion requires interactive
confirmation or `--force` to proceed. Exits with an error if the secret or
version does not exist.

**Renaming a secret:**

```shell
arkana secret rename <secret-name> <new-secret-name>
```

Renames all version files of the secret in `$HOME/.arkana/secrets/`.

### Step 11 — Interactive mode (TUI) `Planned`

Run the tool without arguments to launch a terminal user interface (TUI) for
browsing, decrypting, editing, and re-encrypting stored secrets.

```shell
arkana
```
