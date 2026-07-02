mod cli;
mod io;
mod password;

use crate::cli::{CliArgs, SubCommand};
use crate::io::{read_input, resolve_path, write_output};
use crate::password::read_password;
use arkana::EncryptParams;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli_args = CliArgs::parse();
    let cwd = match cli_args.cwd {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    match cli_args.command {
        Some(SubCommand::Encrypt {
            io,
            encoding,
            kdf,
            cipher,
        }) => {
            let input_file = resolve_path(&cwd, io.input_file)?;
            let output_file = resolve_path(&cwd, io.output_file)?;
            let password_file = resolve_path(&cwd, io.password_file)?;
            let encrypt_params = EncryptParams {
                data: read_input(input_file, &cwd)?,
                password: read_password(password_file)?,
                kdf: kdf.into(),
                cipher: cipher.into(),
            };
            let output = arkana::encrypt(encrypt_params, io.format.into_output(encoding))?;
            write_output(&output, output_file, &cwd)?;
        }
        Some(SubCommand::Decrypt { io }) => {
            let input_file = resolve_path(&cwd, io.input_file)?;
            let output_file = resolve_path(&cwd, io.output_file)?;
            let password_file = resolve_path(&cwd, io.password_file)?;
            let data = read_input(input_file, &cwd)?;
            let decrypted_text =
                arkana::decrypt(&data, io.format.into(), &read_password(password_file)?)?;
            write_output(&decrypted_text, output_file, &cwd)?;
        }
        Some(SubCommand::Convert {
            from_format,
            to_format,
            encoding,
            input_file,
            output_file,
        }) => {
            let input_file = resolve_path(&cwd, input_file)?;
            let output_file = resolve_path(&cwd, output_file)?;
            let data = read_input(input_file, &cwd)?;
            let output =
                arkana::convert(&data, from_format.into(), to_format.into_output(encoding))?;
            write_output(&output, output_file, &cwd)?;
        }
        None => {
            return Err(anyhow::anyhow!("No command specified"));
        }
    }
    Ok(())
}
