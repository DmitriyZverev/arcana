mod crypto;
mod password;

use crate::crypto::{EncryptParams, EncryptedContainer, decrypt, encrypt};
use crate::password::read_password;
use clap::{Args, Parser, Subcommand};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Args)]
struct SubCommandArgs {
    #[arg(long)]
    password_file: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Encrypts data from standard input and writes encrypted data to standard output
    Encrypt(SubCommandArgs),
    /// Decrypts data from standard input and writes decrypted data to standard output
    Decrypt(SubCommandArgs),
}

#[derive(Parser, Debug)]
struct CliArgs {
    #[command(subcommand)]
    command: Option<SubCommand>,
}

fn read_stdin() -> Result<Vec<u8>, std::io::Error> {
    let mut stdin = std::io::stdin();
    let mut data = Vec::new();
    stdin.read_to_end(&mut data)?;
    Ok(data)
}

fn write_stdout(data: &[u8]) -> Result<(), std::io::Error> {
    std::io::stdout().write_all(data)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();
    match cli_args.command {
        Some(SubCommand::Encrypt(SubCommandArgs { password_file })) => {
            let encrypt_params = EncryptParams::new(read_stdin()?, read_password(password_file)?);
            let encrypted_container = encrypt(encrypt_params)?;
            write_stdout(serde_yaml::to_string(&encrypted_container)?.as_bytes())?;
        }
        Some(SubCommand::Decrypt(SubCommandArgs { password_file })) => {
            let stdin = read_stdin()?;
            let encrypted_container = serde_yaml::from_slice::<EncryptedContainer>(&stdin)?;
            let decrypted_text = decrypt(&encrypted_container, &read_password(password_file)?)?;
            write_stdout(&decrypted_text)?;
        }
        None => {
            return Err(anyhow::anyhow!("No command specified"));
        }
    }
    Ok(())
}
