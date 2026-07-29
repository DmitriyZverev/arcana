#![cfg(feature = "deterministic")]

pub mod support;

use support::{ExpectedOutput, SpawnExt, arkana_cmd, create_temp_file, fixtures};

mod format {
    use super::*;

    mod from_yaml {
        use super::*;

        #[test]
        fn to_yaml() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("yaml")
                    .pass_stdin(fixtures::DEFAULT.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_yaml_encoding_base64_from_base16() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("yaml")
                    .arg("--to-format-yaml-encoding")
                    .arg("base64")
                    .pass_stdin(fixtures::FASTEST_BASE16.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_yaml_encoding_base64_alias_from_base16() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("yaml")
                    .arg("--to-yaml-encoding")
                    .arg("base64")
                    .pass_stdin(fixtures::FASTEST_BASE16.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_yaml_encoding_base16_from_lowercase() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("yaml")
                    .arg("--to-format-yaml-encoding")
                    .arg("base16")
                    .pass_stdin(fixtures::FASTEST_BASE16_LOWERCASE.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE16.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_binary() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("binary")
                    .pass_stdin(fixtures::DEFAULT.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_bin()?)
            );
            Ok(())
        }

        #[test]
        fn to_binary_from_base16() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("binary")
                    .pass_stdin(fixtures::FASTEST_BASE16.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE16.envelope_bin()?)
            );
            Ok(())
        }

        #[test]
        fn to_qr() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("yaml")
                    .arg("--to-format")
                    .arg("qr")
                    .pass_stdin(fixtures::DEFAULT.envelope()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_tar()?)
            );
            Ok(())
        }
    }

    mod from_binary {
        use super::*;

        #[test]
        fn to_yaml() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("binary")
                    .arg("--to-format")
                    .arg("yaml")
                    .pass_stdin(fixtures::DEFAULT.envelope_bin()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_yaml_encoding_base16() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("binary")
                    .arg("--to-format")
                    .arg("yaml")
                    .arg("--to-format-yaml-encoding")
                    .arg("base16")
                    .pass_stdin(fixtures::FASTEST_BASE16.envelope_bin()?)?,
                ExpectedOutput::success().stdout(fixtures::FASTEST_BASE16.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_binary() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("binary")
                    .arg("--to-format")
                    .arg("binary")
                    .pass_stdin(fixtures::DEFAULT.envelope_bin()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_bin()?)
            );
            Ok(())
        }

        #[test]
        fn to_qr() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("binary")
                    .arg("--to-format")
                    .arg("qr")
                    .pass_stdin(fixtures::DEFAULT.envelope_bin()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_tar()?)
            );
            Ok(())
        }
    }

    mod from_qr {
        use super::*;

        #[test]
        fn to_yaml() -> anyhow::Result<()> {
            assert_cmd!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("qr")
                    .arg("--to-format")
                    .arg("yaml")
                    .pass_stdin(fixtures::DEFAULT.envelope_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope()?)
            );
            Ok(())
        }

        #[test]
        fn to_binary() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("qr")
                    .arg("--to-format")
                    .arg("binary")
                    .pass_stdin(fixtures::DEFAULT.envelope_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_bin()?)
            );
            Ok(())
        }

        #[test]
        fn to_qr() -> anyhow::Result<()> {
            assert_cmd_binary!(
                arkana_cmd()
                    .arg("convert")
                    .arg("--from-format")
                    .arg("qr")
                    .arg("--to-format")
                    .arg("qr")
                    .pass_stdin(fixtures::DEFAULT.envelope_tar()?)?,
                ExpectedOutput::success().stdout(fixtures::DEFAULT.envelope_tar()?)
            );
            Ok(())
        }
    }
}

mod io_files {
    use super::*;

    #[test]
    fn input_and_output_files() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("convert")
                .arg("--from-format")
                .arg("binary")
                .arg("--to-format")
                .arg("yaml")
                .arg("--input-file")
                .arg(fixtures::DEFAULT.envelope_bin_file_path())
                .arg("--output-file")
                .arg(output_file.path())
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }

    #[test]
    fn input_and_output_files_short_alias() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("convert")
                .arg("-f")
                .arg("binary")
                .arg("-t")
                .arg("yaml")
                .arg("-i")
                .arg(fixtures::DEFAULT.envelope_bin_file_path())
                .arg("-o")
                .arg(output_file.path())
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }

    #[test]
    fn input_and_output_files_long_alias() -> anyhow::Result<()> {
        let output_file = create_temp_file("")?;
        assert_cmd!(
            arkana_cmd()
                .arg("convert")
                .arg("--from")
                .arg("binary")
                .arg("--to")
                .arg("yaml")
                .arg("--input")
                .arg(fixtures::DEFAULT.envelope_bin_file_path())
                .arg("--output")
                .arg(output_file.path())
                .output()?,
            ExpectedOutput::success()
        );
        assert_file!(output_file.path(), fixtures::DEFAULT.envelope()?);
        Ok(())
    }
}
