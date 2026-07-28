# Roadmap

_This document outlines the planned development for `arkana`, progressively
expanding the interface from basic stdin/stdout piping to a full interactive
experience._

_Upcoming features are listed by priority (highest first), not by delivery
order. A `Depends on` line marks features that require another upcoming feature
to land first. A feature that has started moves to `In progress` in place,
without changing sections._

## Status legend

| Badge         | Meaning                            |
| ------------- | ---------------------------------- |
| `Done`        | Fully implemented and available    |
| `In progress` | Work has started, not yet complete |
| `Planned`     | Scheduled, work not yet started    |

## Shipped

### Basic stdin/stdout interface `Done`

Encrypt and decrypt data via standard input/output streams — the foundation for
all later steps.

### File path arguments `Done`

`--input`/`--output` flags as an alternative to stream redirection.

### Encryption parameters `Done`

Flags to override KDF and cipher settings per-invocation (`--kdf-type`,
`--kdf-argon2-*`, `--cipher-type`).

### Encoding field `Done`

`encoding` field in the YAML envelope (`--encoding base16|base32|base64`)
controlling representation of binary values (`salt`, `nonce`, `tag`,
`ciphertext`).

### Binary container format `Done`

`--format binary` — CBOR-encoded envelope as a compact alternative to YAML.

### Format conversion `Done`

`convert` command — transforms an envelope between formats without decryption.

### QR code format `Done`

`--format qr` — envelope as one or more QR code images in a TAR archive, for
physical/paper backups. Each QR symbol encodes a versioned, indexed, checksummed
binary fragment of the CBOR-encoded envelope, so that split containers can be
reassembled and verified regardless of symbol order.

## Upcoming

### PDF format (`--format pdf`) `In progress`

Depends on: QR code format.

Add `pdf` as a new value for the `--format` flag on `encrypt`, `decrypt`, and
`convert` commands. The PDF serves as a physical backup — it contains embedded
raster QR code images (same binary payload format as `--format qr`) and a
formatted human-readable representation of the envelope fields.

```shell
# Encrypt directly to PDF:
arkana encrypt --format pdf \
               --output backup.pdf \
               --input decrypted.txt
arkana encrypt --format pdf \
               --format-pdf-title "My Secret" \
               --format-pdf-timestamp "2024-01-01T00:00:00Z" \
               --output backup.pdf \
               --input decrypted.txt

# Decrypt from PDF:
arkana decrypt --format pdf \
               --input backup.pdf \
               --output decrypted.txt

# Convert existing envelope to PDF:
arkana convert --from-format yaml \
               --to-format pdf \
               --input envelope.yml \
               --output backup.pdf
arkana convert --from-format yaml \
               --to-format pdf \
               --to-format-pdf-title "My Secret" \
               --to-format-pdf-timestamp "2024-01-01T00:00:00Z" \
               --input envelope.yml \
               --output backup.pdf

# Convert from PDF to another format:
arkana convert --from-format pdf \
               --to-format yaml \
               --input backup.pdf \
               --output envelope.yml
```

`--format-pdf-title` sets an optional title shown in the PDF header. If omitted,
the title is left empty. The same flag is available on `convert --to-format pdf`
as `--to-format-pdf-title`.

`--format-pdf-timestamp` accepts an ISO 8601 datetime string and sets the
timestamp shown in the PDF header. If omitted, the current UTC time is used. The
same flag is available on `convert --to-format pdf` as
`--to-format-pdf-timestamp`.

**PDF layout:**

Each page has a header and a body. The header contains the title, SHA-256
checksum of the CBOR-encoded envelope, page number, timestamp, and arkana
version. The body contains up to 4 fragment rows. Fragments are laid out in
document order: encryption parameters fragments first, then ciphertext
fragments. Each fragment row contains a QR code (same binary payload format as
`--format qr`) and a human-readable representation of the fragment data.

**Decoding from PDF:**

`decrypt --format pdf` and `convert --from-format pdf` extract all embedded
raster images from the PDF file and scan each one for QR codes using the same
logic as `--format qr`. Only raster XObjects are considered — vector graphics
are ignored. The header and human-readable data panels are not parsed and have
no effect on decoding.

### Named secret storage `Planned`

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

### Configuration file `Planned`

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

CLI flags (encryption parameters) take precedence over config file values.

### Interactive mode (TUI) `Planned`

Depends on: Named secret storage.

Run the tool without arguments to launch a terminal user interface (TUI) for
browsing, decrypting, editing, and re-encrypting stored secrets.

```shell
arkana
```
