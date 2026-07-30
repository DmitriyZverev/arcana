#![cfg(not(feature = "deterministic"))]

pub mod support;

use support::{ExpectedOutput, SpawnExt, arkana_cmd, fixtures};

#[test]
fn encrypt_decrypt_default() -> anyhow::Result<()> {
    let plaintext = fixtures::DEFAULT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}

#[test]
fn encrypt_decrypt_with_binary_format() -> anyhow::Result<()> {
    let plaintext = fixtures::DEFAULT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--format")
        .arg("binary")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--format")
        .arg("binary")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}

#[test]
fn encrypt_decrypt_with_encoding_base16() -> anyhow::Result<()> {
    let plaintext = fixtures::DEFAULT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--format-yaml-encoding")
        .arg("base16")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}

#[test]
fn encrypt_decrypt_with_encoding_base32() -> anyhow::Result<()> {
    let plaintext = fixtures::DEFAULT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--format-yaml-encoding")
        .arg("base32")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}

#[test]
fn encrypt_decrypt_with_qr_format() -> anyhow::Result<()> {
    let plaintext = fixtures::DEFAULT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--format")
        .arg("qr")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--format")
        .arg("qr")
        .arg("--password-file")
        .arg(fixtures::DEFAULT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}

#[test]
fn encrypt_decrypt_with_qr_format_empty_text() -> anyhow::Result<()> {
    let plaintext = fixtures::EMPTY_TEXT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--format")
        .arg("qr")
        .arg("--password-file")
        .arg(fixtures::EMPTY_TEXT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--format")
        .arg("qr")
        .arg("--password-file")
        .arg(fixtures::EMPTY_TEXT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}

#[test]
fn encrypt_decrypt_with_qr_format_long_text() -> anyhow::Result<()> {
    let plaintext = fixtures::LONG_TEXT.plaintext()?;
    let encrypt_output = arkana_cmd()
        .arg("encrypt")
        .arg("--format")
        .arg("qr")
        .arg("--password-file")
        .arg(fixtures::LONG_TEXT.password_file_path())
        .pass_stdin(plaintext.clone())?;
    assert_eq!(encrypt_output.status.code(), Some(0));
    let decrypt_output = arkana_cmd()
        .arg("decrypt")
        .arg("--format")
        .arg("qr")
        .arg("--password-file")
        .arg(fixtures::LONG_TEXT.password_file_path())
        .pass_stdin(encrypt_output.stdout)?;
    assert_cmd!(decrypt_output, ExpectedOutput::success().stdout(plaintext));
    Ok(())
}
