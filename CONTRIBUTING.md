# Contributing

## Prerequisites

- Rust `1.93.1` or later: https://rustup.rs
- `rustfmt` and `clippy` components: `rustup component add rustfmt clippy`

## Development workflow

Before submitting a PR, ensure the following checks pass:

1. Run tests:

   ```bash
   cargo test --features deterministic
   ```

2. Run linter and static analysis:

   ```bash
   cargo clippy --all-targets --all-features -- -D warnings -W clippy::all
   ```

3. Format the code:

   ```bash
   cargo fmt --all
   ```

## Tests

The `deterministic` feature must be enabled when running tests. It replaces random salt/nonce generation with fixed
values to make snapshots reproducible.

To run all tests:

```bash
cargo test --features deterministic
```

To run a single test:

```bash
cargo test --features deterministic <test_name>
```

Prefer integration tests over unit tests to ensure the behavior of the compiled binary is validated from the user's
perspective. All integration tests live in `tests/*`.

## Commit conventions

- Use imperative mood in commit messages (e.g., "Add feature X" instead of "Added feature X").
- Use `git commit --fixup` and `git rebase -i` to clean up commit history during development.
- Each pull request must be squashed into a single commit before merging (`1 PR = 1 commit`).
- Use the following pattern for commit messages:
  ```
  <type>: <short description>
  
  <full description (if needed)>
  ```
  where `<type>` indicates the **impact on the production artifact** (the
  compiled binary distributed to users):
    - `MAJOR` — breaking changes in the **public API** of the
      program.

      Examples:
        - Removing or renaming CLI commands, arguments, or flags
        - Changing output format in an incompatible way
        - Changing configuration format incompatibly
        - Changing exit codes
        - Any change that requires users or scripts to update
    - `MINOR` — backward-compatible additions to the **public API**.

      Examples:
        - Adding new CLI commands or flags
        - Adding new optional configuration fields
        - Extending the output format in a backward-compatible way
        - Deprecating CLI options
    - `PATCH` — changes affecting **the production artifact** without
      modifying its public API.

      Examples:
        - Bug fixes
        - Internal refactoring
        - Performance improvements
        - Dependency updates (`[dependencies]`)
        - Internal implementation changes
    - `OTHER` — changes that **do not affect the production artifact** or its public API.

      Examples:
        - Tests
        - Examples
        - Documentation
        - CI/CD configuration
        - Repository infrastructure
        - Development tooling
        - Updates to `[dev-dependencies]`

  If a change includes multiple aspects (e.g., a bug fix and test updates),
  choose the type based on the **highest impact on the production artifact**.

## Version determination

Release versions must be determined from commit history since
the last release:

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
- Keep each PR focused on a single change.
- If you want to make multiple changes, submit multiple PRs.
- If you need to refactor code, do it in a separate PR before making the feature changes.
- Include tests for new features and bug fixes.
- Update `README.md` if the change affects user-facing behavior.
- Ensure all CI checks pass.
- Ensure the PR description clearly describes the problem and solution.
