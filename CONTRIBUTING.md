# Contributing

## Prerequisites

- Rust `1.88.0` or later: https://rustup.rs
- `rustfmt` and `clippy` components: `rustup component add rustfmt clippy`
- `dprint`: `cargo install dprint`

## Development workflow

Before submitting a PR, ensure the following checks pass:

1. Run tests:

   ```bash
   cargo test && cargo test --features deterministic
   ```

2. Run linter and static analysis:

   ```bash
   cargo clippy --all-targets --all-features -- -D warnings -W clippy::all
   ```

3. Format the code:

   ```bash
   dprint fmt
   ```

## Tests

To run all tests:

```bash
cargo test && cargo test --features deterministic
```

The `deterministic` feature must be enabled when running snapshot tests. It
replaces random salt/nonce generation with fixed values to make snapshots
reproducible.

To run a single test:

```bash
cargo test <test_name>

# In case of deterministic tests:
cargo test --features deterministic <test_name>
```

Prefer integration tests over unit tests to ensure the behavior of the compiled
binary is validated from the user's perspective. All integration tests live in
`tests/*`.

## Code coverage

Prerequisites:

- `llvm-tools-preview` component:
  ```bash
  rustup component add llvm-tools-preview
  ```
- `cargo-llvm-cov`:
  ```bash
  cargo install cargo-llvm-cov
  ```

To generate an HTML coverage report locally:

```bash
cargo llvm-cov clean --workspace && \
cargo llvm-cov --no-report && \
cargo llvm-cov --no-report --features deterministic && \
cargo llvm-cov report --html --open
```

## Dev utilities (`xtask`)

The `xtask` package contains dev utilities for generating test fixtures and
working with internal formats. Run them via:

```bash
cargo xtask <command> [args...]
```

## Commit conventions

Use the following pattern for commit messages:

```text
<impact>: <short description>

<full description (if needed)>

<tags (if needed)>
```

where:

- `<impact>` indicates the **impact on the production artifact** (the compiled
  binary distributed to users):
  - `MAJOR` — breaking changes in the **public API** of the program.

    Examples:
    - Removing or renaming CLI commands, arguments, or flags
    - Changing the output format in an incompatible way
    - Changing configuration format incompatibly
    - Changing exit codes
    - Any change that requires users or scripts to update
  - `MINOR` — backward-compatible additions to the **public API**.

    Examples:
    - Adding new CLI commands or flags
    - Adding new optional configuration fields
    - Extending the output format in a backward-compatible way
    - Deprecating CLI options
  - `PATCH` — changes affecting **the production artifact** without modifying
    its public API.

    Examples:
    - Bug fixes
    - Internal refactoring
    - Performance improvements
    - Dependency updates (`[dependencies]`)
    - Internal implementation changes
  - `OTHER` — changes that **do not affect the production artifact** or its
    public API.

    Examples:
    - Tests
    - Examples
    - Documentation
    - CI/CD configuration
    - Repository infrastructure
    - Development tooling
    - Updates to `[dev-dependencies]`
- `<short description>` - a concise, imperative-mood summary of the change
  (e.g., "Add feature X").
- `<full description>` - additional context explaining what changed and why, if
  the short description alone is not enough to understand the change.
- `<tags>` - extra tagged data about the change, if needed. Possible tags:
  - `BREAKING_CHANGE` - required for `MAJOR` commits; describes what breaks for
    users and how to migrate (e.g., a before/after comparison of an incompatible
    format change).
  - `PR` - the pull request number this commit originates from (e.g.,
    `PR: #44`).

  Example:
  ```text
  ...

  BREAKING_CHANGE:
  Existing YAML envelopes must be updated manually.

  Before:

  <code>
  kdf:
      type: argon2
      ...
      salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
  </code>

  After:

  <code>
  encoding: base64
  kdf:
      type: argon2
      ...
  salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
  </code>

  PR: #44
  ```

If a change includes multiple aspects (e.g., a bug fix and test updates), choose
the `<impact>` value with the **highest impact on the production artifact**.

Limit commit message line length to 72 characters, excluding formatted blocks
(e.g., code blocks, tables) and links.

## Version determination

Release versions must be determined from commit history since the last release:

```
if any MAJOR commits → major version bump
else if any MINOR commits → minor version bump
else if any PATCH commits → patch version bump
else → no release
```

`OTHER` commits do not affect versioning.

Releases are created from the `main` branch after version determination.

## Pull requests

- Target the `main` branch.
- Each pull request must be squashed into a single commit before merging
  (`1 PR = 1 commit`).
- Use `git commit --fixup` and `git rebase -i` to clean up commit history during
  development.
- Keep each PR focused on a single change.
- If you want to make multiple changes, submit multiple PRs.
- If you need to refactor code, do it in a separate PR before making the feature
  changes.
- Include tests for new features and bug fixes.
- Update `README.md` if the change affects user-facing behavior.
- Ensure all CI checks pass.
- Ensure the PR description clearly describes the problem and solution.
