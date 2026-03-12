use std::path::PathBuf;

pub struct Fixture {
    name: &'static str,
}

impl Fixture {
    pub fn password_file_path(&self) -> PathBuf {
        self.file_path("password.txt")
    }

    pub fn plaintext_file_path(&self) -> PathBuf {
        self.file_path("plaintext.txt")
    }

    pub fn encrypted_container_file_path(&self) -> PathBuf {
        self.file_path("encrypted_container.yml")
    }

    pub fn plaintext(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(self.plaintext_file_path())
    }

    pub fn encrypted_container(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(self.encrypted_container_file_path())
    }

    fn file_path(&self, file_name: &str) -> PathBuf {
        PathBuf::from("tests/fixtures")
            .join(self.name)
            .join(file_name)
    }
}

pub static SHORT_TEXT: Fixture = Fixture { name: "short_text" };
pub static SHORT_TEXT_LOWER_CASE: Fixture = Fixture {
    name: "short_text_lower_case",
};
pub static LONG_TEXT: Fixture = Fixture { name: "long_text" };
