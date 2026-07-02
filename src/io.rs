use path_absolutize::Absolutize;
use pathdiff::diff_paths;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ReadInputError {
    #[error("Failed to read input file: \"{path}\"")]
    File {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to read from stdin")]
    Stdin(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub(crate) enum WriteOutputError {
    #[error("Failed to write output file: \"{path}\"")]
    File {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to write to stdout")]
    Stdout(#[from] std::io::Error),
}

fn display_path(path: &Path, cwd: &Path) -> String {
    diff_paths(path, cwd)
        .as_deref()
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(crate) fn read_input(path: Option<PathBuf>, cwd: &Path) -> Result<Vec<u8>, ReadInputError> {
    match path {
        Some(path) => fs::read(&path).map_err(|source| ReadInputError::File {
            path: display_path(&path, cwd),
            source,
        }),
        None => {
            let mut data = Vec::new();
            std::io::stdin().read_to_end(&mut data)?;
            Ok(data)
        }
    }
}

pub(crate) fn write_output(
    data: &[u8],
    path: Option<PathBuf>,
    cwd: &Path,
) -> Result<(), WriteOutputError> {
    match path {
        Some(path) => fs::write(&path, data).map_err(|source| WriteOutputError::File {
            path: display_path(&path, cwd),
            source,
        }),
        None => Ok(std::io::stdout().write_all(data)?),
    }
}

pub(crate) fn resolve_path(
    base: &Path,
    path: Option<PathBuf>,
) -> Result<Option<PathBuf>, std::io::Error> {
    match path {
        Some(p) => Ok(Some(p.absolutize_from(base)?.to_path_buf())),
        None => Ok(None),
    }
}
