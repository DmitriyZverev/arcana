mod support;

use std::env::current_dir;
use support::{ExpectedOutput, SpawnExt, arcana_cmd, create_temp_file, fixtures};

#[test]
fn encrypt_short_text() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .pass_stdin(fixtures::SHORT_TEXT.plaintext()?)?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.encrypted_container()?)
    );
    Ok(())
}

#[test]
fn decrypt_short_text() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .pass_stdin(fixtures::SHORT_TEXT.encrypted_container()?)?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.plaintext()?)
    );
    Ok(())
}

#[test]
fn encrypt_long_text() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::LONG_TEXT.password_file_path())
            .pass_stdin(fixtures::LONG_TEXT.plaintext()?)?,
        ExpectedOutput::success().stdout(fixtures::LONG_TEXT.encrypted_container()?)
    );
    Ok(())
}

#[test]
fn decrypt_long_text() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::LONG_TEXT.password_file_path())
            .pass_stdin(fixtures::LONG_TEXT.encrypted_container()?)?,
        ExpectedOutput::success().stdout(fixtures::LONG_TEXT.plaintext()?)
    );
    Ok(())
}

#[test]
fn decrypt_with_salt_and_nonce_in_lower_case() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT_LOWER_CASE.password_file_path())
            .pass_stdin(fixtures::SHORT_TEXT_LOWER_CASE.encrypted_container()?)?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT_LOWER_CASE.plaintext()?)
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_password() -> anyhow::Result<()> {
    let password_file = create_temp_file("invalid_password")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.encrypted_container()?)?,
        ExpectedOutput::failure().stderr("Error: Decryption error\n")
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_type() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr(
            "Error: kdf.type: unknown variant `Argon2`, expected `Argon2id` at line 3 column 25\n"
        )
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_memory() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr("Error: Decryption error\n")
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_iterations() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr("Error: Decryption error\n")
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_kdf_parallelism() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr("Error: Decryption error\n")
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_cipher_type() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr(
            "Error: cipher.type: unknown variant `ChaCha20Poly1304`, expected `ChaCha20Poly1305` at line 9 column 25\n"
        )
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_salt() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr("Error: Decryption error\n")
    );
    Ok(())
}

#[test]
fn try_decrypt_with_invalid_nonce() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
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
            )?,
        ExpectedOutput::failure().stderr("Error: Decryption error\n")
    );
    Ok(())
}

#[test]
fn encrypt_with_input_file() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input-file")
            .arg(fixtures::SHORT_TEXT.plaintext_file_path())
            .output()?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.encrypted_container()?)
    );
    Ok(())
}

#[test]
fn encrypt_with_input_file_short_alias() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("-i")
            .arg(fixtures::SHORT_TEXT.plaintext_file_path())
            .output()?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.encrypted_container()?)
    );
    Ok(())
}

#[test]
fn encrypt_with_input_file_long_alias() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input")
            .arg(fixtures::SHORT_TEXT.plaintext_file_path())
            .output()?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.encrypted_container()?)
    );
    Ok(())
}

#[test]
fn encrypt_with_input_file_and_ignore_stdin() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input-file")
            .arg(fixtures::SHORT_TEXT.plaintext_file_path())
            .pass_stdin("Hello everyone!")?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.encrypted_container()?)
    );
    Ok(())
}

#[test]
fn encrypt_with_output_file() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--output-file")
            .arg(output_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.plaintext()?)?,
        ExpectedOutput::success()
    );
    assert_file!(
        output_file.path(),
        fixtures::SHORT_TEXT.encrypted_container()?
    );
    Ok(())
}

#[test]
fn encrypt_with_output_file_short_alias() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("-o")
            .arg(output_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.plaintext()?)?,
        ExpectedOutput::success()
    );
    assert_file!(
        output_file.path(),
        fixtures::SHORT_TEXT.encrypted_container()?
    );
    Ok(())
}

#[test]
fn encrypt_with_output_file_long_alias() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--output")
            .arg(output_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.plaintext()?)?,
        ExpectedOutput::success()
    );
    assert_file!(
        output_file.path(),
        fixtures::SHORT_TEXT.encrypted_container()?
    );
    Ok(())
}

#[test]
fn encrypt_with_input_and_output_files() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input-file")
            .arg(fixtures::SHORT_TEXT.plaintext_file_path())
            .arg("--output-file")
            .arg(output_file.path())
            .output()?,
        ExpectedOutput::success()
    );
    assert_file!(
        output_file.path(),
        fixtures::SHORT_TEXT.encrypted_container()?
    );
    Ok(())
}

#[test]
fn try_encrypt_with_relative_nonexistent_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg("./nonexistent/path/input.txt")
            .output()?,
        ExpectedOutput::failure().stderr(concat!(
            "Error: Failed to read input file: \"nonexistent/path/input.txt\"\n",
            "\n",
            "Caused by:\n",
            "    No such file or directory (os error 2)\n"
        ))
    );
    Ok(())
}

#[test]
fn try_encrypt_with_absolute_nonexistent_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
            .arg("encrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(current_dir()?.join("nonexistent/path/input.txt"))
            .output()?,
        ExpectedOutput::failure().stderr(concat!(
            "Error: Failed to read input file: \"nonexistent/path/input.txt\"\n",
            "\n",
            "Caused by:\n",
            "    No such file or directory (os error 2)\n"
        ))
    );
    Ok(())
}

#[test]
fn decrypt_with_input_file() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input-file")
            .arg(fixtures::SHORT_TEXT.encrypted_container_file_path())
            .output()?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.plaintext()?)
    );
    Ok(())
}

#[test]
fn decrypt_with_input_file_short_alias() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("-i")
            .arg(fixtures::SHORT_TEXT.encrypted_container_file_path())
            .output()?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.plaintext()?)
    );
    Ok(())
}

#[test]
fn decrypt_with_input_file_long_alias() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input")
            .arg(fixtures::SHORT_TEXT.encrypted_container_file_path())
            .output()?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.plaintext()?)
    );
    Ok(())
}

#[test]
fn decrypt_with_input_file_and_ignore_stdin() -> anyhow::Result<()> {
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input-file")
            .arg(fixtures::SHORT_TEXT.encrypted_container_file_path())
            .pass_stdin("Ignored input")?,
        ExpectedOutput::success().stdout(fixtures::SHORT_TEXT.plaintext()?)
    );
    Ok(())
}

#[test]
fn decrypt_with_output_file() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--output-file")
            .arg(output_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.encrypted_container()?)?,
        ExpectedOutput::success()
    );
    assert_file!(output_file.path(), fixtures::SHORT_TEXT.plaintext()?);
    Ok(())
}

#[test]
fn decrypt_with_output_file_short_alias() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("-o")
            .arg(output_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.encrypted_container()?)?,
        ExpectedOutput::success()
    );
    assert_file!(output_file.path(), fixtures::SHORT_TEXT.plaintext()?);
    Ok(())
}

#[test]
fn decrypt_with_output_file_long_alias() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--output")
            .arg(output_file.path())
            .pass_stdin(fixtures::SHORT_TEXT.encrypted_container()?)?,
        ExpectedOutput::success()
    );
    assert_file!(output_file.path(), fixtures::SHORT_TEXT.plaintext()?);
    Ok(())
}

#[test]
fn decrypt_with_input_and_output_files() -> anyhow::Result<()> {
    let output_file = create_temp_file("")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(fixtures::SHORT_TEXT.password_file_path())
            .arg("--input-file")
            .arg(fixtures::SHORT_TEXT.encrypted_container_file_path())
            .arg("--output-file")
            .arg(output_file.path())
            .output()?,
        ExpectedOutput::success()
    );
    assert_file!(output_file.path(), fixtures::SHORT_TEXT.plaintext()?);
    Ok(())
}

#[test]
fn try_decrypt_with_relative_nonexistent_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg("./nonexistent/path/input.txt")
            .output()?,
        ExpectedOutput::failure().stderr(concat!(
            "Error: Failed to read input file: \"nonexistent/path/input.txt\"\n",
            "\n",
            "Caused by:\n",
            "    No such file or directory (os error 2)\n",
        ))
    );
    Ok(())
}

#[test]
fn try_decrypt_with_absolute_nonexistent_input_file() -> anyhow::Result<()> {
    let password_file = create_temp_file("test_password_123")?;
    assert_cmd!(
        arcana_cmd()
            .arg("decrypt")
            .arg("--password-file")
            .arg(password_file.path())
            .arg("--input-file")
            .arg(current_dir()?.join("nonexistent/path/input.txt"))
            .output()?,
        ExpectedOutput::failure().stderr(concat!(
            "Error: Failed to read input file: \"nonexistent/path/input.txt\"\n",
            "\n",
            "Caused by:\n",
            "    No such file or directory (os error 2)\n",
        ))
    );
    Ok(())
}
