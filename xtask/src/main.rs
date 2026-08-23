use clap::{Parser, Subcommand, ValueEnum};
use image::DynamicImage;
use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
enum TarError {
    #[error("failed to read tar file")]
    Read(#[from] std::io::Error),
    #[error("failed to read tar entry")]
    Entry(#[source] std::io::Error),
    #[error("failed to write tar archive")]
    Write(#[source] std::io::Error),
}

#[derive(Error, Debug)]
enum ImgOpError {
    #[error("failed to decode image")]
    Decode(#[from] image::ImageError),
}

#[derive(Error, Debug)]
enum XTaskError {
    #[error(transparent)]
    Tar(#[from] TarError),
    #[error(transparent)]
    Image(#[from] ImgOpError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Codec(#[from] image::ImageError),
    #[error(transparent)]
    Qr(#[from] qrcode::types::QrError),
    #[error("input tar must contain at least {min} image(s), found {found}")]
    TooFewImages { min: usize, found: usize },
    #[error("index {index} is out of range: input tar has {len} entries (1-based)")]
    IndexOutOfRange { index: usize, len: usize },
}

#[derive(Parser)]
#[command(name = "xtask", about = "Developer tasks for arkana")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Mirrors `qrcode::EcLevel`, implementing `ValueEnum` for CLI parsing.
#[derive(Clone, Copy, ValueEnum)]
enum EcLevel {
    L,
    M,
    Q,
    H,
}

impl From<EcLevel> for qrcode::EcLevel {
    fn from(level: EcLevel) -> Self {
        match level {
            EcLevel::L => qrcode::EcLevel::L,
            EcLevel::M => qrcode::EcLevel::M,
            EcLevel::Q => qrcode::EcLevel::Q,
            EcLevel::H => qrcode::EcLevel::H,
        }
    }
}

/// Pixel format of the rendered QR code image.
#[derive(Clone, Copy, ValueEnum)]
enum PixelFormat {
    Grayscale,
    Rgba,
}

#[derive(Subcommand)]
enum Commands {
    /// Combine QR images from TAR into a single PNG grid (default 2 cols)
    JoinQr {
        input: String,
        output: String,
        #[arg(default_value_t = 2)]
        cols: u32,
    },
    /// Repack TAR with alternating PNG/JPEG images
    MixTar { input: String, output: String },
    /// Remove fragment at index (1-based)
    MissingFragment {
        input: String,
        output: String,
        index: usize,
    },
    /// Insert a file into a TAR at index (1-based), shifting later entries back
    InsertFragment {
        input: String,
        output: String,
        index: usize,
        file: String,
    },
    /// Replace the entry at index (1-based) in a TAR with a file
    ReplaceFragment {
        input: String,
        output: String,
        index: usize,
        file: String,
    },
    /// Generate a blank white image
    BlankImage { output: String },
    /// Generate a TAR archive with no entries
    EmptyTar { output: String },
    /// Unpack images from TAR into a directory
    UnpackTar { input: String, output_dir: String },
    /// Convert PNG image to JPEG
    PngToJpeg { input: String, output: String },
    /// Rotate a whole PNG, growing the canvas and filling with white
    RotateImage {
        input: String,
        output: String,
        degrees: f32,
    },
    /// Pack QR codes in a diamond-shaped grid, then rotate 45° into a dense square
    DiamondQr { input: String, output: String },
    /// Generate a single QR code image with a specific version, EC level, pixel scale, and pixel format
    GenQr {
        data: String,
        output: String,
        /// QR code version, 1-40 (higher = more modules, more capacity)
        #[arg(long, default_value_t = 10)]
        version: i16,
        /// Error correction level
        #[arg(long, value_enum, default_value_t = EcLevel::M)]
        ec_level: EcLevel,
        /// Pixels per module (module = one QR code "dot")
        #[arg(long, default_value_t = 10)]
        pixels_per_module: u32,
        /// Pixel format of the output image
        #[arg(long, value_enum, default_value_t = PixelFormat::Grayscale)]
        pixel_format: PixelFormat,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::JoinQr {
            input,
            output,
            cols,
        } => gen_combined_qr(&input, &output, cols)?,
        Commands::MixTar { input, output } => gen_mixed_tar(&input, &output)?,
        Commands::MissingFragment {
            input,
            output,
            index,
        } => gen_missing_fragment(&input, &output, index)?,
        Commands::InsertFragment {
            input,
            output,
            index,
            file,
        } => gen_insert_fragment(&input, &output, index, &file)?,
        Commands::ReplaceFragment {
            input,
            output,
            index,
            file,
        } => gen_replace_fragment(&input, &output, index, &file)?,
        Commands::BlankImage { output } => gen_blank_image(&output)?,
        Commands::EmptyTar { output } => gen_empty_tar(&output)?,
        Commands::UnpackTar { input, output_dir } => cmd_unpack_tar(&input, &output_dir)?,
        Commands::PngToJpeg { input, output } => cmd_png_to_jpeg(&input, &output)?,
        Commands::RotateImage {
            input,
            output,
            degrees,
        } => gen_rotated_image(&input, &output, degrees)?,
        Commands::DiamondQr { input, output } => gen_diamond_qr(&input, &output)?,
        Commands::GenQr {
            data,
            output,
            version,
            ec_level,
            pixels_per_module,
            pixel_format,
        } => gen_qr(
            &data,
            &output,
            version,
            ec_level,
            pixels_per_module,
            pixel_format,
        )?,
    }
    Ok(())
}

fn unpack_tar_bytes(tar_path: &str) -> Result<Vec<Vec<u8>>, TarError> {
    let tar_data = std::fs::read(tar_path)?;
    let mut archive = tar::Archive::new(tar_data.as_slice());
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let mut entry = entry.map_err(TarError::Entry)?;
        let mut data = Vec::new();
        entry.read_to_end(&mut data).map_err(TarError::Entry)?;
        entries.push(data);
    }
    Ok(entries)
}

fn unpack_tar_images(tar_path: &str) -> Result<Vec<DynamicImage>, XTaskError> {
    unpack_tar_bytes(tar_path)?
        .into_iter()
        .map(|data| image::load_from_memory(&data).map_err(|e| ImgOpError::Decode(e).into()))
        .collect()
}

fn png_to_jpeg(image: &DynamicImage) -> Result<Vec<u8>, image::ImageError> {
    let mut data = Vec::new();
    image.write_to(
        &mut std::io::Cursor::new(&mut data),
        image::ImageFormat::Jpeg,
    )?;
    Ok(data)
}

fn cmd_unpack_tar(tar_path: &str, output_dir: &str) -> Result<(), XTaskError> {
    std::fs::create_dir_all(output_dir)?;
    let images = unpack_tar_images(tar_path)?;
    for (i, image) in images.iter().enumerate() {
        let path = format!("{}/{:05}.png", output_dir, i + 1);
        image.save(&path)?;
    }
    println!("Unpacked {} images to {output_dir}", images.len());
    Ok(())
}

fn cmd_png_to_jpeg(input: &str, output: &str) -> Result<(), XTaskError> {
    let image = image::open(input)?;
    let data = png_to_jpeg(&image)?;
    std::fs::write(output, &data)?;
    println!("Converted {input} to {output}");
    Ok(())
}

fn gen_combined_qr(tar_path: &str, png_path: &str, cols: u32) -> Result<(), XTaskError> {
    use image::RgbaImage;
    let images = unpack_tar_images(tar_path)?;
    if images.is_empty() {
        return Err(XTaskError::TooFewImages { min: 1, found: 0 });
    }
    let rows = (images.len() as u32).div_ceil(cols);
    let cell_width = images.iter().map(|img| img.width()).max().unwrap_or(0);
    let cell_height = images.iter().map(|img| img.height()).max().unwrap_or(0);
    let total_width = cell_width * cols;
    let total_height = cell_height * rows;
    let mut combined =
        RgbaImage::from_pixel(total_width, total_height, image::Rgba([255, 255, 255, 255]));
    for (i, img) in images.iter().enumerate() {
        let col = (i as u32) % cols;
        let row = (i as u32) / cols;
        let x = col * cell_width;
        let y = row * cell_height;
        image::imageops::overlay(&mut combined, &img.to_rgba8(), x.into(), y.into());
    }
    DynamicImage::ImageRgba8(combined).save(png_path)?;
    println!(
        "Generated {png_path} ({total_width}x{total_height}, {cols}x{rows} grid, {} images)",
        images.len()
    );
    Ok(())
}

fn tar_append(
    archive: &mut tar::Builder<Vec<u8>>,
    name: &str,
    data: &[u8],
) -> Result<(), TarError> {
    let mut header = tar::Header::new_gnu();
    header.set_size(data.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    archive
        .append_data(&mut header, name, data)
        .map_err(TarError::Write)?;
    Ok(())
}

fn gen_mixed_tar(input: &str, output: &str) -> Result<(), XTaskError> {
    let entries = unpack_tar_bytes(input)?;
    let mut archive = tar::Builder::new(Vec::new());
    for (i, png_data) in entries.iter().enumerate() {
        let (ext, data) = if i % 2 == 0 {
            ("png", png_data.clone())
        } else {
            let img = image::load_from_memory(png_data).map_err(ImgOpError::Decode)?;
            ("jpg", png_to_jpeg(&img)?)
        };
        let name = format!("{:05}.{ext}", i + 1);
        tar_append(&mut archive, &name, &data)?;
    }
    let tar_data = archive.into_inner().map_err(TarError::Write)?;
    std::fs::write(output, &tar_data)?;
    println!(
        "Generated {output} ({} images, mixed PNG/JPEG)",
        entries.len()
    );
    Ok(())
}

fn gen_missing_fragment(input: &str, output: &str, skip_index: usize) -> Result<(), XTaskError> {
    let entries = unpack_tar_bytes(input)?;
    if skip_index == 0 || skip_index > entries.len() {
        return Err(XTaskError::IndexOutOfRange {
            index: skip_index,
            len: entries.len(),
        });
    }
    let mut archive = tar::Builder::new(Vec::new());
    for (pos, data) in entries
        .iter()
        .enumerate()
        .filter(|(i, _)| i + 1 != skip_index)
        .map(|(_, data)| data)
        .enumerate()
    {
        let name = format!("{:05}.png", pos + 1);
        tar_append(&mut archive, &name, data)?;
    }
    let tar_data = archive.into_inner().map_err(TarError::Write)?;
    std::fs::write(output, &tar_data)?;
    println!(
        "Generated {output} ({} entries, skipped index {skip_index})",
        entries.len() - 1
    );
    Ok(())
}

/// File extension to use for a TAR entry name, taken from `path`'s own extension
/// (defaulting to `png` if it has none), so a foreign/non-image file keeps its
/// real extension instead of being mislabeled as `.png`.
fn entry_extension(path: &str) -> &str {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png")
}

fn gen_insert_fragment(
    input: &str,
    output: &str,
    index: usize,
    file: &str,
) -> Result<(), XTaskError> {
    let entries = unpack_tar_bytes(input)?;
    if index == 0 || index > entries.len() + 1 {
        return Err(XTaskError::IndexOutOfRange {
            index,
            len: entries.len() + 1,
        });
    }
    let inserted_data = std::fs::read(file)?;
    let inserted_ext = entry_extension(file);

    let mut archive = tar::Builder::new(Vec::new());
    let mut pos = 0;
    for (i, data) in entries.iter().enumerate() {
        if i + 1 == index {
            pos += 1;
            let name = format!("{pos:05}.{inserted_ext}");
            tar_append(&mut archive, &name, &inserted_data)?;
        }
        pos += 1;
        let name = format!("{pos:05}.png");
        tar_append(&mut archive, &name, data)?;
    }
    if index == entries.len() + 1 {
        pos += 1;
        let name = format!("{pos:05}.{inserted_ext}");
        tar_append(&mut archive, &name, &inserted_data)?;
    }

    let tar_data = archive.into_inner().map_err(TarError::Write)?;
    std::fs::write(output, &tar_data)?;
    println!(
        "Generated {output} ({} entries, inserted {file} at index {index})",
        entries.len() + 1
    );
    Ok(())
}

fn gen_replace_fragment(
    input: &str,
    output: &str,
    index: usize,
    file: &str,
) -> Result<(), XTaskError> {
    let entries = unpack_tar_bytes(input)?;
    if index == 0 || index > entries.len() {
        return Err(XTaskError::IndexOutOfRange {
            index,
            len: entries.len(),
        });
    }
    let replacement_data = std::fs::read(file)?;
    let replacement_ext = entry_extension(file);

    let mut archive = tar::Builder::new(Vec::new());
    for (i, data) in entries.iter().enumerate() {
        if i + 1 == index {
            let name = format!("{:05}.{replacement_ext}", i + 1);
            tar_append(&mut archive, &name, &replacement_data)?;
        } else {
            let name = format!("{:05}.png", i + 1);
            tar_append(&mut archive, &name, data)?;
        }
    }

    let tar_data = archive.into_inner().map_err(TarError::Write)?;
    std::fs::write(output, &tar_data)?;
    println!(
        "Generated {output} ({} entries, replaced index {index} with {file})",
        entries.len()
    );
    Ok(())
}

fn gen_blank_image(output: &str) -> Result<(), image::ImageError> {
    let img = image::GrayImage::from_pixel(100, 100, image::Luma([255u8]));
    DynamicImage::ImageLuma8(img).save(output)?;
    println!("Generated {output} (100x100 blank white)");
    Ok(())
}

fn gen_empty_tar(output: &str) -> Result<(), XTaskError> {
    let archive = tar::Builder::new(Vec::new());
    let tar_data = archive.into_inner().map_err(TarError::Write)?;
    std::fs::write(output, &tar_data)?;
    println!("Generated {output} (0 entries)");
    Ok(())
}

fn gen_rotated_image(
    input_path: &str,
    output_path: &str,
    degrees: f32,
) -> Result<(), image::ImageError> {
    use image::Rgba;
    use imageproc::geometric_transformations::{
        Border, Interpolation, rotate_about_center_no_crop,
    };

    let image = image::open(input_path)?.to_rgba8();
    let theta = degrees.to_radians();
    let white = Rgba([255u8, 255, 255, 255]);
    let rotated = rotate_about_center_no_crop(
        &image,
        theta,
        Interpolation::Nearest,
        Border::Constant(white),
    );

    let (width, height) = rotated.dimensions();
    DynamicImage::ImageRgba8(rotated).save(output_path)?;
    println!("Generated {output_path} ({width}x{height}, rotated {degrees}°)");
    Ok(())
}

/// Number of diamonds a `rows x cols` brick-laid grid holds: even rows (0, 2, ...)
/// hold `cols` diamonds each, odd rows (1, 3, ...) hold `cols - 1` — the odd rows
/// are shifted right by half a diamond so they don't overhang the square canvas,
/// which leaves room for one fewer diamond.
fn brick_grid_capacity(rows: u32, cols: u32) -> u32 {
    let even_rows = rows.div_ceil(2);
    let odd_rows = rows / 2;
    even_rows * cols + odd_rows * cols.saturating_sub(1)
}

/// Fewest rows needed, at a given `cols`, for a brick-laid grid to hold `count` diamonds.
fn rows_needed(count: usize, cols: u32) -> u32 {
    let mut rows = 1;
    while (brick_grid_capacity(rows, cols) as usize) < count {
        rows += 1;
    }
    rows
}

/// Chooses a rows x cols shape whose rendered canvas comes out as square as
/// possible in pixels, using `diamond_side` as the unit (both canvas dimensions
/// scale linearly with it, so the unit cancels out): width is `cols`, height is
/// `(rows - 1) / 2 + 1`. `cols` is searched around the unconstrained estimate
/// `sqrt(count / 2)` for the value whose `rows_needed` brings width and height
/// closest together.
fn square_canvas_shape(count: usize) -> (u32, u32) {
    let pixel_gap = |rows: u32, cols: u32| {
        let width = cols as f64;
        let height = (rows.saturating_sub(1)) as f64 / 2.0 + 1.0;
        (width - height).abs()
    };

    let estimate = ((count as f64) / 2.0).sqrt().ceil().max(1.0) as u32;
    let mut best = (rows_needed(count, estimate), estimate);
    for cols in estimate.saturating_sub(2).max(1)..=estimate + 2 {
        let candidate = (rows_needed(count, cols), cols);
        if pixel_gap(candidate.0, candidate.1) < pixel_gap(best.0, best.1) {
            best = candidate;
        }
    }
    best
}

/// Packs each QR code — rotated 45° individually around its own center — into a
/// brick-laid (checkerboard) diamond grid: rows are spaced `diamond_side/2` apart
/// vertically, and every other row is offset `diamond_side/2` horizontally, so each
/// diamond touches its four neighbors at their vertices with no gaps or overlap.
/// Rows and columns are chosen so the rendered canvas comes out as close to
/// square in pixels as possible; any trailing slots in the last row are left
/// empty.
fn gen_diamond_qr(tar_path: &str, png_path: &str) -> Result<(), XTaskError> {
    use image::{Rgba, RgbaImage};
    use imageproc::geometric_transformations::{
        Border, Interpolation, rotate_about_center_no_crop,
    };

    let images = unpack_tar_images(tar_path)?;
    if images.is_empty() {
        return Err(XTaskError::TooFewImages { min: 1, found: 0 });
    }
    let (rows, cols) = square_canvas_shape(images.len());

    let theta = std::f32::consts::FRAC_PI_4;
    let white = Rgba([255u8, 255, 255, 255]);
    let transparent = Rgba([255u8, 255, 255, 0]);
    let diamonds: Vec<_> = images
        .iter()
        .map(|img| {
            rotate_about_center_no_crop(
                &img.to_rgba8(),
                theta,
                Interpolation::Nearest,
                Border::Constant(transparent),
            )
        })
        .collect();
    // The step between neighboring slot centers must match the ROTATED diamond's own
    // size, not the pre-rotation QR image size — otherwise slots sit closer together
    // than the diamonds are wide, and neighbors overlap instead of merely touching.
    let diamond_side = diamonds.first().map(|d| d.width()).unwrap_or(0);
    let half = diamond_side / 2;

    // Even rows span [0, cols*diamond_side]; odd rows are shifted right by `half`
    // and hold one fewer diamond, so they span [half, (cols-1)*diamond_side + half]
    // — strictly inside the even rows' span. The even-row span is therefore the
    // canvas width, with no extra `half` margin on the right.
    let canvas_width = cols * diamond_side;
    let canvas_height = (rows.saturating_sub(1)) * half + diamond_side;
    let mut canvas = RgbaImage::from_pixel(canvas_width, canvas_height, white);

    // Even rows hold `cols` diamonds starting at x=0; odd rows are shifted right by
    // `half` and hold only `cols - 1`, so they stay within the same canvas width.
    let mut placed = 0usize;
    'rows: for row in 0..rows {
        let row_offset = if row % 2 == 1 { half } else { 0 };
        let row_len = if row % 2 == 1 {
            cols.saturating_sub(1)
        } else {
            cols
        };
        for col in 0..row_len {
            let Some(diamond) = diamonds.get(placed) else {
                break 'rows;
            };
            let x = col * diamond_side + row_offset;
            let y = row * half;
            image::imageops::overlay(&mut canvas, diamond, x.into(), y.into());
            placed += 1;
        }
    }

    let (width, height) = canvas.dimensions();
    DynamicImage::ImageRgba8(canvas).save(png_path)?;
    println!(
        "Generated {png_path} ({width}x{height}, {} codes in a {rows}x{cols} brick-laid diamond grid, each rotated 45°)",
        images.len()
    );
    Ok(())
}

fn gen_qr(
    data: &str,
    output: &str,
    version: i16,
    ec_level: EcLevel,
    pixels_per_module: u32,
    pixel_format: PixelFormat,
) -> Result<(), XTaskError> {
    use image::{Luma, Rgba};

    let code = qrcode::QrCode::with_version(
        data.as_bytes(),
        qrcode::Version::Normal(version),
        ec_level.into(),
    )
    .map_err(XTaskError::Qr)?;

    let (width, height, image) = match pixel_format {
        PixelFormat::Grayscale => {
            let img = code
                .render::<Luma<u8>>()
                .module_dimensions(pixels_per_module, pixels_per_module)
                .build();
            let (width, height) = img.dimensions();
            (width, height, DynamicImage::ImageLuma8(img))
        }
        PixelFormat::Rgba => {
            let img = code
                .render::<Rgba<u8>>()
                .module_dimensions(pixels_per_module, pixels_per_module)
                .build();
            let (width, height) = img.dimensions();
            (width, height, DynamicImage::ImageRgba8(img))
        }
    };

    image.save(output)?;
    println!(
        "Generated {output} ({width}x{height}, version {version}, {pixels_per_module}px/module, {} bytes of data)",
        data.len()
    );
    Ok(())
}
