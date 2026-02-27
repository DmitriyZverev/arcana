use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::io::Write;
use std::process::Command;

fn create_arcana() -> Command {
    Command::new(get_cargo_bin("arcana"))
}

fn create_password_file(password: &str) -> Result<tempfile::NamedTempFile, std::io::Error> {
    let mut password_file = tempfile::NamedTempFile::new()?;
    write!(password_file, "{}", password)?;
    Ok(password_file)
}

#[test]
fn encrypt_short_text() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin("Hello world!"),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----
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
        ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==

        ----- stderr -----

        "###
    );
    Ok(())
}

#[test]
fn decrypt_short_text() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
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
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----
        Hello world!
        ----- stderr -----

        "###
    );
    Ok(())
}

#[test]
fn encrypt_long_text() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent venenatis eleifend nisi id sagittis. Nam vitae consectetur purus."
            ),
        @r###"
    success: true
    exit_code: 0
    ----- stdout -----
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

    ----- stderr -----

    "###
    );
    Ok(())
}

#[test]
fn decrypt_long_text() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
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
                "
            ),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent venenatis eleifend nisi id sagittis. Nam vitae consectetur purus.
        ----- stderr -----

        "###
    );
    Ok(())
}

#[test]
fn decrypt_with_salt_and_nonce_in_lower_case() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131072
                  iterations: 4
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1305
                salt: 1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b
                nonce: 0a0a0a0a0a0a0a0a0a0a0a0a
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----
        Hello world!
        ----- stderr -----

        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_password() -> anyhow::Result<()> {
    let password_file = create_password_file("invalid_password")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
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
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Decryption error
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_type() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2
                  version: 19
                  memory: 131072
                  iterations: 4
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1305
                salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: kdf.type: unknown variant `Argon2`, expected `Argon2id` at line 3 column 25
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_memory() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131071
                  iterations: 4
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1305
                salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Decryption error
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_iterations() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131072
                  iterations: 1
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1305
                salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Decryption error
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_parallelism() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131072
                  iterations: 4
                  parallelism: 1
                cipher:
                  type: ChaCha20Poly1305
                salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Decryption error
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_cipher_type() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131072
                  iterations: 4
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1304
                salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: cipher.type: unknown variant `ChaCha20Poly1304`, expected `ChaCha20Poly1305` at line 9 column 25
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_salt() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131072
                  iterations: 4
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1305
                salt: 0B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 0A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Decryption error
        "###
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_nonce() -> anyhow::Result<()> {
    let password_file = create_password_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(
                "
                kdf:
                  type: Argon2id
                  version: 19
                  memory: 131072
                  iterations: 4
                  parallelism: 4
                cipher:
                  type: ChaCha20Poly1305
                salt: 1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B1B
                nonce: 1A0A0A0A0A0A0A0A0A0A0A0A
                ciphertext: RmuSIEhbLyex+iTUh1yYEdQ5IHcvz3UL7W+ZHQ==
                "
            ),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Decryption error
        "###
    );
    Ok(())
}
