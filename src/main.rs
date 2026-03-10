mod crypto;
mod password;

use crate::crypto::{EncryptParams, EncryptedContainer, decrypt, encrypt};
use crate::password::read_password;
use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Args)]
struct SubCommandArgs {
    #[arg(long)]
    password_file: Option<PathBuf>,
    /// Read input from a file instead of standard input
    #[arg(long, short = 'i', alias = "input")]
    input_file: Option<PathBuf>,
    /// Write output to a file instead of standard output
    #[arg(long, short = 'o', alias = "output")]
    output_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Encrypts data from stdin or a file and writes encrypted data to stdout or a file
    Encrypt(SubCommandArgs),
    /// Decrypts data from stdin or a file and writes decrypted data to stdout or a file
    Decrypt(SubCommandArgs),
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: Option<SubCommand>,
}

fn read_input(path: Option<PathBuf>) -> anyhow::Result<Vec<u8>> {
    match path {
        Some(p) => {
            fs::read(&p).with_context(|| format!("Failed to read input file: {}", p.display()))
        }
        None => {
            let mut data = Vec::new();
            std::io::stdin().read_to_end(&mut data)?;
            Ok(data)
        }
    }
}

fn write_output(data: &[u8], path: Option<PathBuf>) -> anyhow::Result<()> {
    match path {
        Some(p) => fs::write(&p, data)
            .with_context(|| format!("Failed to write output file: {}", p.display())),
        None => Ok(std::io::stdout().write_all(data)?),
    }
}

fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();
    match cli_args.command {
        Some(SubCommand::Encrypt(SubCommandArgs {
            password_file,
            input_file,
            output_file,
        })) => {
            let encrypt_params =
                EncryptParams::new(read_input(input_file)?, read_password(password_file)?);
            let encrypted_container = encrypt(encrypt_params)?;
            write_output(
                serde_yaml::to_string(&encrypted_container)?.as_bytes(),
                output_file,
            )?;
        }
        Some(SubCommand::Decrypt(SubCommandArgs {
            password_file,
            input_file,
            output_file,
        })) => {
            let data = read_input(input_file)?;
            let encrypted_container = serde_yaml::from_slice::<EncryptedContainer>(&data)?;
            let decrypted_text = decrypt(&encrypted_container, &read_password(password_file)?)?;
            write_output(&decrypted_text, output_file)?;
        }
        None => {
            return Err(anyhow::anyhow!("No command specified"));
        }
    }
    Ok(())
}
