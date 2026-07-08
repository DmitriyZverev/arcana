# Principles

Design decisions in `arkana` must follow these principles.

## Cryptography

### 1. Proven standards

All algorithms and constructions must be widely reviewed and standardized. No
custom cryptographic schemes.

### 2. Secure defaults

Default parameters must provide strong security without any configuration. A
user who specifies no flags must get protection aligned with current
recommendations.

## Envelope

### 1. Readability

The container must use a well-known plain-text format that a person can read,
understand, copy, and transcribe from paper, that an engineer can identify on
sight, and that a machine can parse without ambiguity.

### 2. Self-documentation

The container must include everything needed to decrypt it besides the password
— algorithms, parameters, and all cryptographic artifacts. No hidden logic or
implicit defaults. Any engineer must be able to decrypt the container by reading
its structure alone.

## Encoding selection

Every supported binary encoding must satisfy **all** the following.

### 1. Human readability

The encoding must be practical for reading from paper and typing by hand.

- The alphabet must avoid visually ambiguous characters where possible (e.g.
  `0`/`O`, `1`/`l`/`I`).
- Case-insensitive encodings are preferred over case-sensitive ones, as case
  errors are the most common mistake during manual transcription.

An encoding may relax these preferences if the loss in readability is justified
by a significant gain in compactness (see principle 2).

### 2. Compactness

The encoding must provide a reasonable size overhead relative to the raw binary
data. Readability and compactness are inversely correlated — a larger alphabet
encodes more densely but introduces case sensitivity and ambiguous characters.
Each supported encoding must occupy a distinct point on this trade-off curve;
there must be no other eligible encoding that is strictly better on both axes.

### 3. Format safety

The alphabet must not contain characters with special meaning in the container
format, so that manually typed encoded data cannot be confused with format
syntax.

### 4. Formal specification

The encoding must be defined in a published RFC or a stable industry-standard
specification with comparable levels of adoption and documentation. The goal is
to ensure that any engineer in the future can find the encoding rules without
access to the original arkana source code.
