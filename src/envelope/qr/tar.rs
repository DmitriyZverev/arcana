use std::io::Read;
use tar::{Archive, Builder, Header};

pub(super) struct File {
    pub(super) path: String,
    pub(super) content: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum PackError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub(super) fn pack(files: &[File]) -> Result<Vec<u8>, PackError> {
    let mut archive = Builder::new(Vec::new());
    for file in files {
        let mut header = Header::new_gnu();
        header.set_size(file.content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append_data(&mut header, &file.path, file.content.as_slice())?;
    }
    archive.finish()?;
    Ok(archive.into_inner()?)
}

#[derive(Debug, thiserror::Error)]
pub enum UnpackError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub(super) fn unpack(bytes: &[u8]) -> Result<Vec<File>, UnpackError> {
    let mut archive = Archive::new(bytes);
    let mut files = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_string_lossy().into_owned();
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        files.push(File { path, content });
    }
    Ok(files)
}
