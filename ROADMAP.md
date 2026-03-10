# Roadmap

_This document outlines the planned development stages for `arcana`._

_Each step builds on the previous one, progressively expanding the interface from basic
stdin/stdout piping to a full interactive experience_

## Status legend

| Badge         | Meaning                            |
|---------------|------------------------------------|
| `Done`        | Fully implemented and available    |
| `In progress` | Work has started, not yet complete |
| `Planned`     | Scheduled, work not yet started    |

## Milestones

### Step 1 — Basic stdin/stdout interface `Done`

Encrypt and decrypt data using standard input/output streams. This is the minimal
viable interface and serves as the foundation for all later steps.

```shell
arcana encrypt < decrypted.txt > encrypted.yml
arcana decrypt < encrypted.yml > decrypted.txt
```

### Step 2 — File path arguments `Done`

Add `--input` and `--output` flags as an alternative to stream redirection. Useful when
integrating with scripts or tools that work with file paths directly.

```shell
arcana encrypt --input ./decrypted.txt --output ./encrypted.yml
arcana decrypt --input ./encrypted.yml --output ./decrypted.txt
```

### Step 3 — Named document storage `Planned`

Introduce a document registry stored in `$HOME/.arcana/documents/`. Each encryption
creates a new versioned snapshot of the document, making it possible to track and
restore previous versions.

File naming pattern: `<document-name>.YYYY_MM_DD_HH_mm_ss_fffffffff_<counter>.yml`

**Encrypting a named document:**

```shell
# From stdin:
arcana encrypt --document document-name < ./decrypted.txt

# From file:
arcana encrypt --document document-name --input ./decrypted.txt
```

Both commands write the encrypted result to:
`$HOME/.arcana/documents/document-name.YYYY_MM_DD_HH_mm_ss_fffffffff_0001.yml`

**Decrypting a named document:**

```shell
# To stdout:
arcana decrypt --document document-name > decrypted.txt

# To file:
arcana decrypt --document document-name --output ./decrypted.txt
```

Decryption automatically resolves to the latest version of the document found in
`$HOME/.arcana/documents/`.

### Step 4 — Interactive mode (TUI) `Planned`

Run the tool without arguments to launch a terminal user interface (TUI) for browsing,
decrypting, editing, and re-encrypting stored documents.

```shell
arcana
```
