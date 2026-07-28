#![cfg(feature = "deterministic")]

pub mod support;

use indoc::indoc;
use std::env::current_dir;
use support::{
    ExpectedOutput, SpawnExt, arkana_cmd, create_temp_dir, create_temp_file, create_temp_file_in,
    fixtures, relative_to,
};

mod basic {
    use super::*;

    #[test]
    fn short_text() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .pass_stdin(fixtures::DEFAULT.envelope()?)?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
        );
        Ok(())
    }

    #[test]
    fn long_text() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::LONG_TEXT.password_file_path())
                .pass_stdin(fixtures::LONG_TEXT.envelope()?)?,
            ExpectedOutput::success().stdout(fixtures::LONG_TEXT.plaintext()?)
        );
        Ok(())
    }
}

mod io_files {
    use super::*;

    #[test]
    fn input_file() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input-file")
                .arg(fixtures::DEFAULT.envelope_file_path())
                .output()?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
        );
        Ok(())
    }

    #[test]
    fn input_file_short_alias() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("-i")
                .arg(fixtures::DEFAULT.envelope_file_path())
                .output()?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
        );
        Ok(())
    }

    #[test]
    fn input_file_long_alias() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input")
                .arg(fixtures::DEFAULT.envelope_file_path())
                .output()?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
        );
        Ok(())
    }

    #[test]
    fn input_file_and_ignore_stdin() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input-file")
                .arg(fixtures::DEFAULT.envelope_file_path())
                .pass_stdin("Ignored input")?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
        );
        Ok(())
    }

    #[test]
    fn output_file() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--output-file")
                .arg(output_file.path())
                .pass_stdin(fixtures::DEFAULT.envelope()?)?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.plaintext()?);
        Ok(())
    }

    #[test]
    fn output_file_short_alias() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("-o")
                .arg(output_file.path())
                .pass_stdin(fixtures::DEFAULT.envelope()?)?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.plaintext()?);
        Ok(())
    }

    #[test]
    fn output_file_long_alias() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--output")
                .arg(output_file.path())
                .pass_stdin(fixtures::DEFAULT.envelope()?)?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.plaintext()?);
        Ok(())
    }

    #[test]
    fn input_and_output_files() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input-file")
                .arg(fixtures::DEFAULT.envelope_file_path())
                .arg("--output-file")
                .arg(output_file.path())
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.plaintext()?);
        Ok(())
    }

    #[test]
    fn cwd_and_relative_input_and_output_files() -> anyhow::Result<()> {
        let current_dir = create_temp_dir()?;
        let password_file = create_temp_file_in(&current_dir, &fixtures::DEFAULT.password()?)?;
        let input_file = create_temp_file_in(&current_dir, &fixtures::DEFAULT.envelope()?)?;
        let output_file = create_temp_file_in(&current_dir, "")?;
        let relative_password_file = relative_to(&password_file, &current_dir)?;
        let relative_input_file = relative_to(&input_file, &current_dir)?;
        let relative_output_file = relative_to(&output_file, &current_dir)?;
        assert_cmd!(
            arkana_cmd()
                .arg("--cwd")
                .arg(current_dir.path())
                .arg("decrypt")
                .arg("--password-file")
                .arg(relative_password_file)
                .arg("--input-file")
                .arg(relative_input_file)
                .arg("--output-file")
                .arg(relative_output_file)
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.plaintext()?);
        Ok(())
    }

    #[test]
    fn err_relative_nonexistent_input_file() -> anyhow::Result<()> {
        let password_file = create_temp_file("test_password_123")?;
        #[cfg(unix)]
        let expected_stderr = indoc! {"
            Error: Failed to read input file: \"nonexistent/path/input.txt\"

            Caused by:
                No such file or directory (os error 2)
        "};
        #[cfg(windows)]
        let expected_stderr = indoc! {"
            Error: Failed to read input file: \"nonexistent\\path\\input.txt\"

            Caused by:
                The system cannot find the path specified. (os error 3)
        "};
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(password_file.path())
                .arg("--input-file")
                .arg("./nonexistent/path/input.txt")
                .output()?,
            ExpectedOutput::failure().stderr(expected_stderr)
        );
        Ok(())
    }

    #[test]
    fn err_absolute_nonexistent_input_file() -> anyhow::Result<()> {
        let password_file = create_temp_file("test_password_123")?;
        #[cfg(unix)]
        let expected_stderr = indoc! {"
            Error: Failed to read input file: \"nonexistent/path/input.txt\"

            Caused by:
                No such file or directory (os error 2)
        "};
        #[cfg(windows)]
        let expected_stderr = indoc! {"
            Error: Failed to read input file: \"nonexistent\\path\\input.txt\"

            Caused by:
                The system cannot find the path specified. (os error 3)
        "};
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(password_file.path())
                .arg("--input-file")
                .arg(current_dir()?.join("nonexistent/path/input.txt"))
                .output()?,
            ExpectedOutput::failure().stderr(expected_stderr)
        );
        Ok(())
    }
}

mod params {
    use super::*;

    #[test]
    fn err_invalid_password() -> anyhow::Result<()> {
        let password_file = create_temp_file("invalid_password")?;
        assert_cmd!(
            arkana_cmd()
                .arg("decrypt")
                .arg("--password-file")
                .arg(password_file.path())
                .pass_stdin(fixtures::DEFAULT.envelope()?)?,
            ExpectedOutput::failure().stderr(indoc! {"
                Error: Decryption failed
            "})
        );
        Ok(())
    }

    mod kdf {
        use super::*;

        #[test]
        fn err_invalid_type() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .pass_stdin(indoc! {"
                        encoding: base64
                        params:
                          kdf:
                            type: argon3
                            algorithm: argon2id
                            version: 19
                            memory: 131072
                            iterations: 4
                            parallelism: 4
                            salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                          cipher:
                            type: ChaCha20Poly1305
                            nonce: CgoKCgoKCgoKCgoK
                            tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                        ciphertext: RmuSIEhbLyex+iTU
                    "})?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: params.kdf.type: unknown variant `argon3`, expected `argon2` at line 4 column 11
                "})
            );
            Ok(())
        }

        mod argon2 {
            use super::*;

            #[test]
            fn fastest() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::FASTEST.password_file_path())
                        .pass_stdin(fixtures::FASTEST.envelope()?)?,
                    ExpectedOutput::success().stdout(fixtures::FASTEST.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn algorithm_argon2i() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I.password_file_path())
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I.envelope()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn algorithm_argon2d() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D.password_file_path())
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D.envelope()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn version_16() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_VERSION_16.password_file_path())
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_VERSION_16.envelope()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_VERSION_16.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn memory_65536() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_MEMORY_65536.password_file_path())
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_MEMORY_65536.envelope()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_MEMORY_65536.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn iterations_1() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_ITERATIONS_1.password_file_path())
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_ITERATIONS_1.envelope()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_ITERATIONS_1.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn parallelism_1() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_PARALLELISM_1.password_file_path())
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_PARALLELISM_1.envelope()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_PARALLELISM_1.plaintext()?)
                );
                Ok(())
            }

            #[test]
            fn err_invalid_algorithm() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .pass_stdin(indoc! {"
                            encoding: base64
                            params:
                              kdf:
                                type: argon2
                                algorithm: argon2
                                version: 19
                                memory: 131072
                                iterations: 4
                                parallelism: 4
                                salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                              cipher:
                                type: ChaCha20Poly1305
                                nonce: CgoKCgoKCgoKCgoK
                                tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                            ciphertext: RmuSIEhbLyex+iTU
                        "})?,
                    ExpectedOutput::failure().stderr(indoc! {"
                        Error: params: unknown variant `argon2`, expected one of `argon2i`, `argon2d`, `argon2id` at line 3 column 3
                    "})
                );
                Ok(())
            }

            #[test]
            fn err_invalid_memory() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .pass_stdin(indoc! {"
                            encoding: base64
                            params:
                              kdf:
                                type: argon2
                                algorithm: argon2id
                                version: 19
                                memory: 131071
                                iterations: 4
                                parallelism: 4
                                salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                              cipher:
                                type: ChaCha20Poly1305
                                nonce: CgoKCgoKCgoKCgoK
                                tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                            ciphertext: RmuSIEhbLyex+iTU
                        "})?,
                    ExpectedOutput::failure().stderr(indoc! {"
                        Error: Decryption failed
                    "})
                );
                Ok(())
            }

            #[test]
            fn err_invalid_iterations() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .pass_stdin(indoc! {"
                            encoding: base64
                            params:
                              kdf:
                                type: argon2
                                algorithm: argon2id
                                version: 19
                                memory: 131072
                                iterations: 1
                                parallelism: 4
                                salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                              cipher:
                                type: ChaCha20Poly1305
                                nonce: CgoKCgoKCgoKCgoK
                                tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                            ciphertext: RmuSIEhbLyex+iTU
                        "})?,
                    ExpectedOutput::failure().stderr(indoc! {"
                        Error: Decryption failed
                    "})
                );
                Ok(())
            }

            #[test]
            fn err_invalid_parallelism() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .pass_stdin(indoc! {"
                            encoding: base64
                            params:
                              kdf:
                                type: argon2
                                algorithm: argon2id
                                version: 19
                                memory: 131072
                                iterations: 4
                                parallelism: 1
                                salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                              cipher:
                                type: ChaCha20Poly1305
                                nonce: CgoKCgoKCgoKCgoK
                                tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                            ciphertext: RmuSIEhbLyex+iTU
                        "})?,
                    ExpectedOutput::failure().stderr(indoc! {"
                        Error: Decryption failed
                    "})
                );
                Ok(())
            }

            #[test]
            fn err_invalid_salt() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .pass_stdin(indoc! {"
                                    encoding: base64
                                    params:
                                      kdf:
                                        type: argon2
                                        algorithm: argon2id
                                        version: 19
                                        memory: 131072
                                        iterations: 4
                                        parallelism: 4
                                        salt: CxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                                      cipher:
                                        type: ChaCha20Poly1305
                                        nonce: CgoKCgoKCgoKCgoK
                                        tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                                    ciphertext: RmuSIEhbLyex+iTU
                                "})?,
                    ExpectedOutput::failure().stderr(indoc! {"
                                Error: Decryption failed
                            "})
                );
                Ok(())
            }

            #[test]
            fn err_invalid_salt_length() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("decrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .pass_stdin(indoc! {"
                                    encoding: base64
                                    params:
                                      kdf:
                                        type: argon2
                                        algorithm: argon2id
                                        version: 19
                                        memory: 131072
                                        iterations: 4
                                        parallelism: 4
                                        salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGw==
                                      cipher:
                                        type: ChaCha20Poly1305
                                        nonce: CgoKCgoKCgoKCgoK
                                        tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                                    ciphertext: RmuSIEhbLyex+iTU
                                "})?,
                    ExpectedOutput::failure().stderr(indoc! {"
                                Error: Decoding error: Invalid length: expected 32, actual 31

                                Caused by:
                                    Invalid length: expected 32, actual 31
                            "})
                );
                Ok(())
            }
        }
    }

    mod cipher {
        use super::*;

        #[test]
        fn err_cipher_type() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .pass_stdin(indoc! {"
                        encoding: base64
                        params:
                          kdf:
                            type: argon2
                            algorithm: argon2id
                            version: 19
                            memory: 131072
                            iterations: 4
                            parallelism: 4
                            salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                          cipher:
                            type: ChaCha20Poly1304
                            nonce: CgoKCgoKCgoKCgoK
                            tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                        ciphertext: RmuSIEhbLyex+iTU
                    "})?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: params.cipher.type: unknown variant `ChaCha20Poly1304`, expected `ChaCha20Poly1305` at line 12 column 11
                "})
            );
            Ok(())
        }

        #[test]
        fn err_nonce() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .pass_stdin(indoc! {"
                        encoding: base64
                        params:
                          kdf:
                            type: argon2
                            algorithm: argon2id
                            version: 19
                            memory: 131072
                            iterations: 4
                            parallelism: 4
                            salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                          cipher:
                            type: ChaCha20Poly1305
                            nonce: GgoKCgoKCgoKCgoK
                            tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                        ciphertext: RmuSIEhbLyex+iTU
                    "})?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Decryption failed
                "})
            );
            Ok(())
        }

        #[test]
        fn err_nonce_length() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .pass_stdin(indoc! {"
                        encoding: base64
                        params:
                          kdf:
                            type: argon2
                            algorithm: argon2id
                            version: 19
                            memory: 131072
                            iterations: 4
                            parallelism: 4
                            salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                          cipher:
                            type: ChaCha20Poly1305
                            nonce: CgoKCgoKCgoKCgo=
                            tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                        ciphertext: RmuSIEhbLyex+iTU
                    "})?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Decoding error: Invalid length: expected 12, actual 11

                    Caused by:
                        Invalid length: expected 12, actual 11
                "})
            );
            Ok(())
        }

        #[test]
        fn err_tag_length() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .pass_stdin(indoc! {"
                        encoding: base64
                        params:
                          kdf:
                            type: argon2
                            algorithm: argon2id
                            version: 19
                            memory: 131072
                            iterations: 4
                            parallelism: 4
                            salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                          cipher:
                            type: ChaCha20Poly1305
                            nonce: CgoKCgoKCgoKCgoK
                            tag: h1yYEdQ5IHcvz3UL7W+Z
                        ciphertext: RmuSIEhbLyex+iTU
                    "})?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Decoding error: Invalid length: expected 16, actual 15

                    Caused by:
                        Invalid length: expected 16, actual 15
                "})
            );
            Ok(())
        }
    }
}

mod format {
    use super::*;

    mod yaml {
        use super::*;

        #[test]
        fn plain() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("yaml")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn encoding_base16() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(fixtures::FASTEST_BASE16.password_file_path())
                    .pass_stdin(fixtures::FASTEST_BASE16.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE16.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn encoding_base16_lowercase() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(fixtures::FASTEST_BASE16_LOWERCASE.password_file_path())
                    .pass_stdin(fixtures::FASTEST_BASE16_LOWERCASE.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE16_LOWERCASE.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn encoding_base32() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(fixtures::FASTEST_BASE32.password_file_path())
                    .pass_stdin(fixtures::FASTEST_BASE32.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE32.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn err_invalid_encoding_value() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .pass_stdin(indoc! {"
                        encoding: base58
                        params:
                          kdf:
                            type: argon2
                            algorithm: argon2id
                            version: 19
                            memory: 131072
                            iterations: 4
                            parallelism: 4
                            salt: GxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxsbGxs=
                          cipher:
                            type: ChaCha20Poly1305
                            nonce: CgoKCgoKCgoKCgoK
                            tag: h1yYEdQ5IHcvz3UL7W+ZHQ==
                        ciphertext: RmuSIEhbLyex+iTU
                    "})?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: encoding: unknown variant `base58`, expected one of `base16`, `base32`, `base64` at line 1 column 11
                "})
            );
            Ok(())
        }

        #[test]
        fn err_binary_envelope() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("yaml")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_bin()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: control characters are not allowed at position 6
                "})
            );
            Ok(())
        }
    }

    mod binary {
        use super::*;

        #[test]
        fn plain() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_bin()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn err_invalid_params_kdf_type() -> anyhow::Result<()> {
            let mut data = fixtures::DEFAULT.envelope_bin()?;
            // replace "argon2" with "argon3" in CBOR params to make kdf.type invalid
            let pos = data.windows(6).position(|w| w == b"argon2").unwrap();
            data[pos + 5] = b'3';
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: CBOR error: Semantic(None, \"unknown variant `argon3`, expected `argon2`\")

                    Caused by:
                        Semantic(None, \"unknown variant `argon3`, expected `argon2`\")
                "})
            );
            Ok(())
        }

        #[test]
        fn err_header_too_short() -> anyhow::Result<()> {
            let mut data = vec![0u8; 10];
            data[..6].copy_from_slice(b"arkana");
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Input too short: expected 15 bytes, got 10
                "})
            );
            Ok(())
        }

        #[test]
        fn err_invalid_magic() -> anyhow::Result<()> {
            let mut data = fixtures::DEFAULT.envelope_bin()?;
            data[..6].copy_from_slice(b"foobar");
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Invalid format: not an arkana binary envelope
                "})
            );
            Ok(())
        }

        #[test]
        fn err_unsupported_version() -> anyhow::Result<()> {
            let mut data = fixtures::DEFAULT.envelope_bin()?;
            data[6] = 2;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Unsupported version: 2
                "})
            );
            Ok(())
        }

        #[test]
        fn err_params_too_short() -> anyhow::Result<()> {
            let data = fixtures::DEFAULT.envelope_bin()?;
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data[..16].as_ref())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Input too short: expected 208 bytes, got 16
                "})
            );
            Ok(())
        }

        #[test]
        fn err_trailing_bytes() -> anyhow::Result<()> {
            let mut data = fixtures::DEFAULT.envelope_bin()?;
            // layout: [ magic: 6B ][ version: 1B ][ params_len: 4B ][ ciphertext_len: 4B ][ params: CBOR ][ ciphertext ]
            let params_len = u32::from_be_bytes(data[7..11].try_into()?) as usize;
            data[7..11].copy_from_slice(&((params_len as u32 + 1).to_be_bytes()));
            data.insert(15 + params_len, 0xff);
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Trailing bytes after params
                "})
            );
            Ok(())
        }

        #[test]
        fn err_ciphertext_too_short() -> anyhow::Result<()> {
            let mut data = fixtures::DEFAULT.envelope_bin()?;
            let ciphertext_len = u32::from_be_bytes(data[11..15].try_into()?) as usize;
            data[11..15].copy_from_slice(&((ciphertext_len as u32 + 1).to_be_bytes()));
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Input too short: expected 221 bytes, got 220
                "})
            );
            Ok(())
        }

        #[test]
        fn err_ciphertext_trailing_bytes() -> anyhow::Result<()> {
            let mut data = fixtures::DEFAULT.envelope_bin()?;
            let ciphertext_len = u32::from_be_bytes(data[11..15].try_into()?) as usize;
            data[11..15].copy_from_slice(&((ciphertext_len as u32 - 1).to_be_bytes()));
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(data.as_slice())?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Trailing bytes after ciphertext
                "})
            );
            Ok(())
        }

        #[test]
        fn err_yaml_envelope() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope()?)?,
                ExpectedOutput::failure().stderr(indoc! {r#"
                    Error: Invalid format: not an arkana binary envelope
                "#})
            );
            Ok(())
        }
    }

    mod qr {
        use super::*;

        #[test]
        fn from_tar() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn long_text_from_tar() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::LONG_TEXT.password_file_path())
                    .pass_stdin(fixtures::LONG_TEXT.envelope_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::LONG_TEXT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn long_text_from_png() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::LONG_TEXT.password_file_path())
                    .pass_stdin(fixtures::LONG_TEXT.envelope_png()?)?,
                ExpectedOutput::success().stdout(fixtures::LONG_TEXT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn from_png() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_png()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn from_jpg() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_jpg()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn from_mixed_tar() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_mixed_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn err_from_tar_with_missing_fragment() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::invalid::missing_fragment_tar()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Missing fragments: [1]
                "})
            );
            Ok(())
        }

        #[test]
        fn err_from_no_qr_png() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::invalid::no_qr_png()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Missing fragments: [1]
                "})
            );
            Ok(())
        }

        #[test]
        fn err_from_empty_tar() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::invalid::empty_tar()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Missing fragments: [1]
                "})
            );
            Ok(())
        }

        #[test]
        fn err_from_non_image_tar() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::invalid::non_image_tar()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: The image format could not be determined
                "})
            );
            Ok(())
        }

        #[test]
        fn from_tar_with_valid_extra_foreign_qrcode() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_with_foreign_qrcode_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn from_tar_with_no_qr_image() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_with_no_qr_image_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.plaintext()?)
            );
            Ok(())
        }

        #[test]
        fn err_from_tar_with_fragment_replaced_by_foreign_qrcode() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(
                        fixtures::invalid::envelope_fragment_replaced_with_foreign_qrcode_tar()?
                    )?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Missing fragments: [2]
                "})
            );
            Ok(())
        }

        #[test]
        fn err_from_foreign_qrcode_png() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::invalid::foreign_qrcode_png()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: Missing fragments: [1]
                "})
            );
            Ok(())
        }

        #[test]
        fn err_yaml_envelope() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: numeric field was not a number: ILE MANU when getting cksum for # Arkana-encrypted envelope (arkana v0.1.0)
                    # Decrypt instructions: https://github.com/DmitriyZverev
                "})
            );
            Ok(())
        }

        #[test]
        fn err_binary_envelope() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("decrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.envelope_bin()?)?,
                ExpectedOutput::failure().stderr(indoc! {"
                    Error: failed to read entire block
                "})
            );
            Ok(())
        }
    }
}
