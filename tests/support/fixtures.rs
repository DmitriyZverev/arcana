use std::path::PathBuf;

const PLACEHOLDERS: &[(&str, &str)] = &[
    ("${CARGO_PKG_VERSION}", env!("CARGO_PKG_VERSION")),
    ("${CARGO_PKG_REPOSITORY}", env!("CARGO_PKG_REPOSITORY")),
];

fn substitute_placeholders(content: String) -> String {
    PLACEHOLDERS
        .iter()
        .fold(content, |content, (placeholder, value)| {
            content.replace(placeholder, value)
        })
}

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

    pub fn envelope_file_path(&self) -> PathBuf {
        self.file_path("envelope.yml")
    }

    pub fn password(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(self.password_file_path())
    }

    pub fn plaintext(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(self.plaintext_file_path())
    }

    pub fn envelope(&self) -> Result<String, std::io::Error> {
        let content = std::fs::read_to_string(self.envelope_file_path())?;
        Ok(substitute_placeholders(content))
    }

    pub fn envelope_bin_file_path(&self) -> PathBuf {
        self.file_path("envelope.bin")
    }

    pub fn envelope_bin(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_bin_file_path())
    }

    pub fn envelope_tar_file_path(&self) -> PathBuf {
        self.file_path("envelope.tar")
    }

    pub fn envelope_tar(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_tar_file_path())
    }

    pub fn envelope_png_file_path(&self) -> PathBuf {
        self.file_path("envelope.png")
    }

    pub fn envelope_png(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_png_file_path())
    }

    pub fn envelope_png_3_col_file_path(&self) -> PathBuf {
        self.file_path("envelope_3_col.png")
    }

    pub fn envelope_png_3_col(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_png_3_col_file_path())
    }

    pub fn envelope_png_4_col_file_path(&self) -> PathBuf {
        self.file_path("envelope_4_col.png")
    }

    pub fn envelope_png_4_col(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_png_4_col_file_path())
    }

    pub fn envelope_png_5_col_file_path(&self) -> PathBuf {
        self.file_path("envelope_5_col.png")
    }

    pub fn envelope_png_5_col(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_png_5_col_file_path())
    }

    pub fn envelope_png_45_deg_file_path(&self) -> PathBuf {
        self.file_path("envelope_45_deg.png")
    }

    pub fn envelope_png_45_deg(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_png_45_deg_file_path())
    }

    pub fn envelope_jpg_file_path(&self) -> PathBuf {
        self.file_path("envelope.jpg")
    }

    pub fn envelope_jpg(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_jpg_file_path())
    }

    pub fn envelope_mixed_tar_file_path(&self) -> PathBuf {
        self.file_path("envelope_mixed.tar")
    }

    pub fn envelope_mixed_tar(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.envelope_mixed_tar_file_path())
    }

    pub fn envelope_with_foreign_qrcode_tar(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.file_path("envelope_with_foreign_qrcode.tar"))
    }

    pub fn envelope_with_no_qr_image_tar(&self) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(self.file_path("envelope_with_no_qr_image.tar"))
    }

    fn file_path(&self, file_name: &str) -> PathBuf {
        PathBuf::from("tests/fixtures")
            .join(self.name)
            .join(file_name)
    }
}

pub static DEFAULT: Fixture = Fixture { name: "default" };
pub static DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2I: Fixture = Fixture {
    name: "default_kdf_argon2_algorithm_argon2i",
};
pub static DEFAULT_KDF_ARGON2_ALGORITHM_ARGON2D: Fixture = Fixture {
    name: "default_kdf_argon2_algorithm_argon2d",
};
pub static DEFAULT_KDF_ARGON2_ITERATIONS_1: Fixture = Fixture {
    name: "default_kdf_argon2_iterations_1",
};
pub static DEFAULT_KDF_ARGON2_MEMORY_65536: Fixture = Fixture {
    name: "default_kdf_argon2_memory_65536",
};
pub static DEFAULT_KDF_ARGON2_PARALLELISM_1: Fixture = Fixture {
    name: "default_kdf_argon2_parallelism_1",
};
pub static DEFAULT_KDF_ARGON2_VERSION_16: Fixture = Fixture {
    name: "default_kdf_argon2_version_16",
};
pub static FASTEST: Fixture = Fixture { name: "fastest" };
pub static FASTEST_BASE16: Fixture = Fixture {
    name: "fastest_base16",
};
pub static FASTEST_BASE32: Fixture = Fixture {
    name: "fastest_base32",
};
pub static FASTEST_BASE16_LOWERCASE: Fixture = Fixture {
    name: "fastest_base16_lowercase",
};
pub static LONG_TEXT: Fixture = Fixture { name: "long_text" };
pub static EMPTY_TEXT: Fixture = Fixture { name: "empty_text" };

pub mod invalid {
    use std::path::PathBuf;

    fn file_path(file_name: &str) -> PathBuf {
        PathBuf::from("tests/fixtures/invalid").join(file_name)
    }

    pub fn missing_fragment_tar() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path("envelope_missing_fragment.tar"))
    }

    pub fn no_qr_png() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path("no_qr.png"))
    }

    pub fn non_image_tar() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path("non_image.tar"))
    }

    pub fn empty_tar() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path("empty.tar"))
    }

    pub fn foreign_qrcode_png() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path("foreign_qrcode.png"))
    }

    pub fn envelope_fragment_replaced_with_foreign_qrcode_tar() -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(file_path(
            "envelope_fragment_replaced_with_foreign_qrcode.tar",
        ))
    }
}
