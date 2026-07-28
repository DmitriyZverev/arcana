#![cfg(feature = "deterministic")]

pub mod support;

use indoc::indoc;
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
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
        );
        Ok(())
    }

    #[test]
    fn long_text() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::LONG_TEXT.password_file_path())
                .pass_stdin(fixtures::LONG_TEXT.plaintext()?)?,
            ExpectedOutput::success().stdout(fixtures::LONG_TEXT.envelope()?)
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
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input-file")
                .arg(fixtures::DEFAULT.plaintext_file_path())
                .output()?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
        );
        Ok(())
    }

    #[test]
    fn input_file_short_alias() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("-i")
                .arg(fixtures::DEFAULT.plaintext_file_path())
                .output()?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
        );
        Ok(())
    }

    #[test]
    fn input_file_long_alias() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input")
                .arg(fixtures::DEFAULT.plaintext_file_path())
                .output()?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
        );
        Ok(())
    }

    #[test]
    fn input_file_and_ignore_stdin() -> anyhow::Result<()> {
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input-file")
                .arg(fixtures::DEFAULT.plaintext_file_path())
                .pass_stdin("Hello everyone!")?,
            ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
        );
        Ok(())
    }

    #[test]
    fn output_file() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--output-file")
                .arg(output_file.path())
                .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }

    #[test]
    fn output_file_short_alias() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("-o")
                .arg(output_file.path())
                .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }

    #[test]
    fn output_file_long_alias() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--output")
                .arg(output_file.path())
                .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }

    #[test]
    fn input_and_output_files() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("encrypt")
                .arg("--password-file")
                .arg(fixtures::DEFAULT.password_file_path())
                .arg("--input-file")
                .arg(fixtures::DEFAULT.plaintext_file_path())
                .arg("--output-file")
                .arg(output_file.path())
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }

    #[test]
    fn cwd_and_relative_input_and_output_files() -> anyhow::Result<()> {
        let current_dir = create_temp_dir()?;
        let password_file = create_temp_file_in(&current_dir, &fixtures::DEFAULT.password()?)?;
        let input_file = create_temp_file_in(&current_dir, &fixtures::DEFAULT.plaintext()?)?;
        let output_file = create_temp_file_in(&current_dir, "")?;
        let relative_password_file = relative_to(&password_file, &current_dir)?;
        let relative_input_file = relative_to(&input_file, &current_dir)?;
        let relative_output_file = relative_to(&output_file, &current_dir)?;
        assert_cmd!(
            arkana_cmd()
                .arg("--cwd")
                .arg(current_dir.path())
                .arg("encrypt")
                .arg("--password-file")
                .arg(relative_password_file)
                .arg("--input-file")
                .arg(relative_input_file)
                .arg("--output-file")
                .arg(relative_output_file)
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
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
                .arg("encrypt")
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
                .arg("encrypt")
                .arg("--password-file")
                .arg(password_file.path())
                .arg("--input-file")
                .arg(std::env::current_dir()?.join("nonexistent/path/input.txt"))
                .output()?,
            ExpectedOutput::failure().stderr(expected_stderr)
        );
        Ok(())
    }
}

mod params {
    use super::*;

    mod kdf {
        use super::*;

        #[test]
        fn err_invalid_type() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .arg("--kdf-type")
                    .arg("invalid")
                    .output()?,
                ExpectedOutput::code(2).stderr(indoc! {"
                        error: invalid value 'invalid' for '--kdf-type <KDF_TYPE>'
                          [possible values: argon2]

                        For more information, try '--help'.
                    "})
            );
            Ok(())
        }

        mod argon2 {
            use super::*;

            #[test]
            fn plain() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT.password_file_path())
                        .arg("--kdf-type")
                        .arg("argon2")
                        .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
                    ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn fastest() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::FASTEST.password_file_path())
                        .arg("--kdf-argon2-memory")
                        .arg("32")
                        .arg("--kdf-argon2-iterations")
                        .arg("1")
                        .arg("--kdf-argon2-parallelism")
                        .arg("4")
                        .pass_stdin(fixtures::FASTEST.plaintext()?)?,
                    ExpectedOutput::success().stdout(fixtures::FASTEST.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn algorithm_argon2i() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I.password_file_path())
                        .arg("--kdf-argon2-algorithm")
                        .arg("argon2i")
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I.plaintext()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn algorithm_argon2d() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D.password_file_path())
                        .arg("--kdf-argon2-algorithm")
                        .arg("argon2d")
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D.plaintext()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn err_invalid_algorithm() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .arg("--kdf-argon2-algorithm")
                        .arg("invalid")
                        .output()?,
                    ExpectedOutput::code(2).stderr(indoc! {"
                            error: invalid value 'invalid' for '--kdf-argon2-algorithm <ALGORITHM>'
                              [possible values: argon2id, argon2i, argon2d]

                            For more information, try '--help'.
                        "})
                );
                Ok(())
            }

            #[test]
            fn version_16() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_VERSION_16.password_file_path())
                        .arg("--kdf-argon2-version")
                        .arg("16")
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_VERSION_16.plaintext()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_VERSION_16.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn err_invalid_version() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(password_file.path())
                        .arg("--kdf-argon2-version")
                        .arg("17")
                        .output()?,
                    ExpectedOutput::code(2).stderr(indoc! {"
                            error: invalid value '17' for '--kdf-argon2-version <VERSION>'
                              [possible values: 16, 19]

                            For more information, try '--help'.
                        "})
                );
                Ok(())
            }

            #[test]
            fn memory_65536() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_MEMORY_65536.password_file_path())
                        .arg("--kdf-argon2-memory")
                        .arg("65536")
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_MEMORY_65536.plaintext()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_MEMORY_65536.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn err_invalid_memory() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                        arkana_cmd()
                            .arg("encrypt")
                            .arg("--password-file")
                            .arg(password_file.path())
                            .arg("--kdf-argon2-memory")
                            .arg("abc")
                            .output()?,
                        ExpectedOutput::code(2).stderr(indoc! {"
                            error: invalid value 'abc' for '--kdf-argon2-memory <MEMORY>': invalid digit found in string

                            For more information, try '--help'.
                        "})
                    );
                Ok(())
            }

            #[test]
            fn iterations_1() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_ITERATIONS_1.password_file_path())
                        .arg("--kdf-argon2-iterations")
                        .arg("1")
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_ITERATIONS_1.plaintext()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_ITERATIONS_1.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn err_invalid_iterations() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                        arkana_cmd()
                            .arg("encrypt")
                            .arg("--password-file")
                            .arg(password_file.path())
                            .arg("--kdf-argon2-iterations")
                            .arg("abc")
                            .output()?,
                        ExpectedOutput::code(2).stderr(indoc! {"
                            error: invalid value 'abc' for '--kdf-argon2-iterations <ITERATIONS>': invalid digit found in string

                            For more information, try '--help'.
                        "})
                    );
                Ok(())
            }

            #[test]
            fn parallelism_1() -> anyhow::Result<()> {
                assert_cmd!(
                    arkana_cmd()
                        .arg("encrypt")
                        .arg("--password-file")
                        .arg(fixtures::DEFAULT_KDF_ARGON2_PARALLELISM_1.password_file_path())
                        .arg("--kdf-argon2-parallelism")
                        .arg("1")
                        .pass_stdin(fixtures::DEFAULT_KDF_ARGON2_PARALLELISM_1.plaintext()?)?,
                    ExpectedOutput::success()
                        .stdout(fixtures::DEFAULT_KDF_ARGON2_PARALLELISM_1.envelope()?)
                );
                Ok(())
            }

            #[test]
            fn err_invalid_parallelism() -> anyhow::Result<()> {
                let password_file = create_temp_file("test_password_123")?;
                assert_cmd!(
                        arkana_cmd()
                            .arg("encrypt")
                            .arg("--password-file")
                            .arg(password_file.path())
                            .arg("--kdf-argon2-parallelism")
                            .arg("abc")
                            .output()?,
                        ExpectedOutput::code(2).stderr(indoc! {"
                            error: invalid value 'abc' for '--kdf-argon2-parallelism <PARALLELISM>': invalid digit found in string

                            For more information, try '--help'.
                        "})
                    );
                Ok(())
            }
        }
    }

    mod cipher {
        use super::*;

        #[test]
        fn cha_cha_20_poly_1305() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .arg("--cipher-type")
                    .arg("ChaCha20Poly1305")
                    .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn err_invalid_type() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .arg("--cipher-type")
                    .arg("invalid")
                    .output()?,
                ExpectedOutput::code(2).stderr(indoc! {"
                    error: invalid value 'invalid' for '--cipher-type <CIPHER_TYPE>'
                      [possible values: ChaCha20Poly1305]

                    For more information, try '--help'.
                "})
            );
            Ok(())
        }
    }
}

mod format {
    use super::*;

    mod binary {
        use super::*;

        #[test]
        fn plain() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--format")
                    .arg("binary")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_bin()?)
            );
            Ok(())
        }
    }

    mod yaml {
        use super::*;

        #[test]
        fn plain() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--format")
                    .arg("yaml")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn encoding_base16() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--encoding")
                    .arg("base16")
                    .arg("--password-file")
                    .arg(fixtures::FASTEST_BASE16.password_file_path())
                    .arg("--kdf-argon2-memory")
                    .arg("32")
                    .arg("--kdf-argon2-iterations")
                    .arg("1")
                    .arg("--kdf-argon2-parallelism")
                    .arg("4")
                    .pass_stdin(fixtures::FASTEST_BASE16.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE16.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn encoding_base32() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--encoding")
                    .arg("base32")
                    .arg("--password-file")
                    .arg(fixtures::FASTEST_BASE32.password_file_path())
                    .arg("--kdf-argon2-memory")
                    .arg("32")
                    .arg("--kdf-argon2-iterations")
                    .arg("1")
                    .arg("--kdf-argon2-parallelism")
                    .arg("4")
                    .pass_stdin(fixtures::FASTEST_BASE32.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE32.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn encoding_base64_matches_default() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--encoding")
                    .arg("base64")
                    .arg("--password-file")
                    .arg(fixtures::FASTEST.password_file_path())
                    .arg("--kdf-argon2-memory")
                    .arg("32")
                    .arg("--kdf-argon2-iterations")
                    .arg("1")
                    .arg("--kdf-argon2-parallelism")
                    .arg("4")
                    .pass_stdin(fixtures::FASTEST.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn err_invalid_encoding() -> anyhow::Result<()> {
            let password_file = create_temp_file("test_password_123")?;
            assert_cmd!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--password-file")
                    .arg(password_file.path())
                    .arg("--encoding")
                    .arg("invalid")
                    .output()?,
                ExpectedOutput::code(2).stderr(indoc! {"
                    error: invalid value 'invalid' for '--encoding <ENCODING>'
                      [possible values: base16, base32, base64]

                    For more information, try '--help'.
                "})
            );
            Ok(())
        }
    }

    mod qr {
        use super::*;

        #[test]
        fn plain() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::DEFAULT.password_file_path())
                    .pass_stdin(fixtures::DEFAULT.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_tar()?)
            );
            Ok(())
        }

        #[test]
        fn long_text() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("encrypt")
                    .arg("--format")
                    .arg("qr")
                    .arg("--password-file")
                    .arg(fixtures::LONG_TEXT.password_file_path())
                    .pass_stdin(fixtures::LONG_TEXT.plaintext()?)?,
                ExpectedOutput::success().stdout(fixtures::LONG_TEXT.envelope_tar()?)
            );
            Ok(())
        }
    }
}
