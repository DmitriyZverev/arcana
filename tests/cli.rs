use insta::assert_snapshot;
use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::io::Write;
use std::process::Command;

fn create_arcana() -> Command {
    Command::new(get_cargo_bin("arcana"))
}

fn create_temp_file(content: &str) -> Result<tempfile::NamedTempFile, std::io::Error> {
    let mut file = tempfile::NamedTempFile::new()?;
    write!(file, "{}", content)?;
    Ok(file)
}

#[test]
fn encrypt_short_text() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("invalid_password")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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
    let password_file = create_temp_file("test_password_123")?;
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

#[test]
fn encrypt_with_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file("Hello world!")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(input_file.path()),
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
fn encrypt_with_input_file_short_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file("Hello world!")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("-i")
            .arg(input_file.path()),
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
fn encrypt_with_input_file_long_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file("Hello world!")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input")
            .arg(input_file.path()),
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
fn encrypt_with_input_file_and_ignore_stdin() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file("Hello world!")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(input_file.path())
            .pass_stdin("Hello everyone!"),
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
fn encrypt_with_output_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--output-file")
            .arg(output_file.path())
            .pass_stdin("Hello world!"),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @r###"
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
        "###
    );
    Ok(())
}

#[test]
fn encrypt_with_output_file_short_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("-o")
            .arg(output_file.path())
            .pass_stdin("Hello world!"),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @r###"
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
        "###
    );
    Ok(())
}

#[test]
fn encrypt_with_output_file_long_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--output")
            .arg(output_file.path())
            .pass_stdin("Hello world!"),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @r###"
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
        "###
    );
    Ok(())
}

#[test]
fn encrypt_with_input_and_output_files() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file("Hello world!")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(input_file.path())
            .arg("--output-file")
            .arg(output_file.path()),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @r###"
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
        "###
    );
    Ok(())
}

#[test]
fn try_encrypt_with_nonexistent_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg("./nonexistent/path/input.txt"),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Failed to read input file: ./nonexistent/path/input.txt

        Caused by:
            No such file or directory (os error 2)

        "###
    );
    Ok(())
}

#[test]
fn decrypt_with_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file(
        "kdf:
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
",
    )?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(input_file.path()),
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
fn decrypt_with_input_file_short_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file(
        "kdf:
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
",
    )?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("-i")
            .arg(input_file.path()),
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
fn decrypt_with_input_file_long_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file(
        "kdf:
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
",
    )?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input")
            .arg(input_file.path()),
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
fn decrypt_with_input_file_and_ignore_stdin() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file(
        "kdf:
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
",
    )?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(input_file.path())
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
                ciphertext: RmuSIEhbPT6m5DmXPseEPcaOYJHC6mINwb5N9E9f1n0=
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
fn decrypt_with_output_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--output-file")
            .arg(output_file.path())
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

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @"Hello world!"
    );
    Ok(())
}

#[test]
fn decrypt_with_output_file_short_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("-o")
            .arg(output_file.path())
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

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @"Hello world!"
    );
    Ok(())
}

#[test]
fn decrypt_with_output_file_long_alias() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--output")
            .arg(output_file.path())
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

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @"Hello world!"
    );
    Ok(())
}

#[test]
fn decrypt_with_input_and_output_files() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let input_file = create_temp_file(
        "kdf:
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
",
    )?;
    let output_file = create_temp_file("")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(input_file.path())
            .arg("--output-file")
            .arg(output_file.path()),
        @r###"
        success: true
        exit_code: 0
        ----- stdout -----

        ----- stderr -----

        "###
    );
    assert_snapshot!(
        std::fs::read_to_string(output_file.path())?,
        @"Hello world!"
    );
    Ok(())
}

#[test]
fn try_decrypt_with_nonexistent_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    let mut arcana = create_arcana();
    assert_cmd_snapshot!(
        arcana
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg("./nonexistent/path/input.txt"),
        @r###"
        success: false
        exit_code: 1
        ----- stdout -----

        ----- stderr -----
        Error: Failed to read input file: ./nonexistent/path/input.txt

        Caused by:
            No such file or directory (os error 2)

        "###
    );
    Ok(())
}
