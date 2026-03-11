pub mod fixtures;
pub mod macros;

use std::io::Write;
use std::process::{Command, Output, Stdio};

pub struct ExpectedOutput {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl ExpectedOutput {
    pub fn success() -> Self {
        Self::code(0)
    }

    pub fn failure() -> Self {
        Self::code(1)
    }

    pub fn code(code: i32) -> Self {
        Self {
            code: Some(code),
            stdout: vec![],
            stderr: vec![],
        }
    }

    pub fn stdout(mut self, stdout: impl Into<Vec<u8>>) -> Self {
        self.stdout = stdout.into();
        self
    }

    pub fn stderr(mut self, stderr: impl Into<Vec<u8>>) -> Self {
        self.stderr = stderr.into();
        self
    }
}

pub trait SpawnExt {
    fn pass_stdin(&mut self, input: impl Into<Vec<u8>>) -> Result<Output, std::io::Error>;
}

impl SpawnExt for Command {
    fn pass_stdin(&mut self, input: impl Into<Vec<u8>>) -> Result<Output, std::io::Error> {
        self.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = self.spawn()?;
        let input = input.into();
        if let Some(mut stdin) = child.stdin.take() {
            std::thread::spawn(move || stdin.write_all(&input).ok());
        }
        child.wait_with_output()
    }
}

pub fn arcana_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_arcana"))
}

pub fn create_temp_file(content: &str) -> Result<tempfile::NamedTempFile, std::io::Error> {
    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(content.as_bytes())?;
    Ok(file)
}
