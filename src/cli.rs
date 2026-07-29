use argon2::{Algorithm, Version};
use arkana::envelope::text::yaml::SerializeParams as YamlSerializeParams;
use arkana::{
    crypto::{CipherParams, KdfParams},
    envelope::Argon2Params,
    format::{InputFormat, OutputFormat},
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::Display;
use std::path::PathBuf;

fn display_value(
    value: &impl ValueEnum,
    formatter: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(
        formatter,
        "{}",
        value.to_possible_value().ok_or(std::fmt::Error)?.get_name()
    )
}

#[derive(Debug, Args)]
pub struct CipherArgs {
    /// Cipher to use for encryption.
    #[arg(long, default_value_t)]
    cipher_type: CipherType,
}

impl From<CipherArgs> for CipherParams {
    fn from(cipher: CipherArgs) -> Self {
        match cipher.cipher_type {
            CipherType::ChaCha20Poly1305 => CipherParams::ChaCha20Poly1305,
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum CipherType {
    #[default]
    #[value(name = "ChaCha20Poly1305")]
    ChaCha20Poly1305,
}

impl Display for CipherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_value(self, f)
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum Argon2Version {
    #[value(name = "16")]
    V16,
    #[default]
    #[value(name = "19")]
    V19,
}

impl Display for Argon2Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_value(self, f)
    }
}

impl From<Argon2Version> for Version {
    fn from(version: Argon2Version) -> Self {
        match version {
            Argon2Version::V16 => Version::V0x10,
            Argon2Version::V19 => Version::V0x13,
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum Argon2Algorithm {
    #[default]
    #[value(name = "argon2id")]
    Argon2id,
    #[value(name = "argon2i")]
    Argon2i,
    #[value(name = "argon2d")]
    Argon2d,
}

impl Display for Argon2Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_value(self, f)
    }
}

impl From<Argon2Algorithm> for Algorithm {
    fn from(algorithm: Argon2Algorithm) -> Self {
        match algorithm {
            Argon2Algorithm::Argon2id => Algorithm::Argon2id,
            Argon2Algorithm::Argon2i => Algorithm::Argon2i,
            Argon2Algorithm::Argon2d => Algorithm::Argon2d,
        }
    }
}

#[derive(Debug, Args)]
pub struct Argon2Args {
    /// Argon2 algorithm to use for key derivation.
    #[arg(long = "kdf-argon2-algorithm", default_value_t)]
    algorithm: Argon2Algorithm,
    /// Argon2 version to use for key derivation.
    #[arg(long = "kdf-argon2-version", default_value_t)]
    version: Argon2Version,
    /// Argon2 memory to use for key derivation.
    #[arg(long = "kdf-argon2-memory", default_value_t = Argon2Params::DEFAULT_MEMORY)]
    memory: u32,
    /// Argon2 iterations to use for key derivation.
    #[arg(long = "kdf-argon2-iterations", default_value_t = Argon2Params::DEFAULT_ITERATIONS)]
    iterations: u32,
    /// Argon2 parallelism to use for key derivation.
    #[arg(long = "kdf-argon2-parallelism", default_value_t = Argon2Params::DEFAULT_PARALLELISM)]
    parallelism: u32,
}

impl From<Argon2Args> for Argon2Params {
    fn from(argon2_args: Argon2Args) -> Self {
        Argon2Params {
            algorithm: argon2_args.algorithm.into(),
            version: argon2_args.version.into(),
            memory: argon2_args.memory,
            iterations: argon2_args.iterations,
            parallelism: argon2_args.parallelism,
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum KdfType {
    #[default]
    #[value(name = "argon2")]
    Argon2,
}

impl Display for KdfType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_value(self, f)
    }
}

#[derive(Debug, Args)]
pub struct KdfArgs {
    /// Key derivation function to use for key derivation.
    #[arg(long, default_value_t)]
    kdf_type: KdfType,
    #[command(flatten)]
    argon2: Argon2Args,
}

impl From<KdfArgs> for KdfParams {
    fn from(kdf: KdfArgs) -> Self {
        match kdf.kdf_type {
            KdfType::Argon2 => KdfParams::Argon2 {
                params: kdf.argon2.into(),
            },
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum Format {
    #[default]
    #[value(name = "yaml")]
    Yaml,
    #[value(name = "binary")]
    Binary,
    #[value(name = "qr")]
    Qr,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_value(self, f)
    }
}

impl From<Format> for InputFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Yaml => InputFormat::Yaml,
            Format::Binary => InputFormat::Binary,
            Format::Qr => InputFormat::Qr,
        }
    }
}

pub struct FormatParams {
    pub yaml: YamlSerializeParams,
}

impl Format {
    pub fn into_output(self, params: FormatParams) -> OutputFormat {
        match self {
            Format::Yaml => OutputFormat::Yaml(params.yaml),
            Format::Binary => OutputFormat::Binary,
            Format::Qr => OutputFormat::Qr,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum Encoding {
    #[value(name = "base16")]
    Base16,
    #[value(name = "base32")]
    Base32,
    #[default]
    #[value(name = "base64")]
    Base64,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_value(self, f)
    }
}

impl From<Encoding> for arkana::envelope::text::Encoding {
    fn from(encoding: Encoding) -> Self {
        match encoding {
            Encoding::Base16 => arkana::envelope::text::Encoding::Base16,
            Encoding::Base32 => arkana::envelope::text::Encoding::Base32,
            Encoding::Base64 => arkana::envelope::text::Encoding::Base64,
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct IoArgs {
    /// Read password from a file instead of prompting for it
    #[arg(long, short = 'p')]
    pub password_file: Option<PathBuf>,
    /// Read input from a file instead of standard input
    #[arg(long, short = 'i', alias = "input")]
    pub input_file: Option<PathBuf>,
    /// Write output to a file instead of standard output
    #[arg(long, short = 'o', alias = "output")]
    pub output_file: Option<PathBuf>,
    /// Envelope format
    #[arg(long, short = 'f', default_value_t)]
    pub format: Format,
}

#[derive(Debug, Args)]
pub(crate) struct FormatYamlArgs {
    /// Binary encoding for YAML format. [default: base64]
    #[arg(long = "format-yaml-encoding")]
    encoding: Option<Encoding>,
}

impl From<FormatYamlArgs> for YamlSerializeParams {
    fn from(args: FormatYamlArgs) -> YamlSerializeParams {
        YamlSerializeParams {
            encoding: args.encoding.map(|e| e.into()),
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct ToFormatYamlArgs {
    /// Binary encoding for YAML format. [default: base64]
    #[arg(long = "to-format-yaml-encoding", alias = "to-yaml-encoding")]
    encoding: Option<Encoding>,
}

impl From<ToFormatYamlArgs> for YamlSerializeParams {
    fn from(args: ToFormatYamlArgs) -> YamlSerializeParams {
        YamlSerializeParams {
            encoding: args.encoding.map(|e| e.into()),
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum SubCommand {
    /// Encrypts data from stdin or a file and writes encrypted data to stdout or a file
    Encrypt {
        #[command(flatten)]
        io: IoArgs,
        #[command(flatten)]
        yaml: FormatYamlArgs,
        #[command(flatten)]
        kdf: KdfArgs,
        #[command(flatten)]
        cipher: CipherArgs,
    },
    /// Decrypts data from stdin or a file and writes decrypted data to stdout or a file
    Decrypt {
        #[command(flatten)]
        io: IoArgs,
    },
    /// Converts an encrypted envelope between formats without decryption
    Convert {
        /// Source envelope format
        #[arg(long, short = 'f', alias = "from")]
        from_format: Format,
        /// Target envelope format
        #[arg(long, short = 't', alias = "to")]
        to_format: Format,
        #[command(flatten)]
        yaml: ToFormatYamlArgs,
        /// Read input from a file instead of standard input
        #[arg(long, short = 'i', alias = "input")]
        input_file: Option<PathBuf>,
        /// Write output to a file instead of standard output
        #[arg(long, short = 'o', alias = "output")]
        output_file: Option<PathBuf>,
    },
}

#[derive(Parser, Debug)]
pub(crate) struct CliArgs {
    /// Working directory for resolving relative paths
    #[arg(long, short = 'C')]
    pub cwd: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}
