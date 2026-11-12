use std::io::Cursor;

use image::{ImageBuffer, ImageError, ImageFormat, Rgb};

pub type RgbBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub(crate) const WHITE_PIXEL: Rgb<u8> = Rgb([255, 255, 255]);
pub(crate) const BLACK_PIXEL: Rgb<u8> = Rgb([0, 0, 0]);
#[cfg(feature = "maze")]
pub(crate) const RED_PIXEL: Rgb<u8> = Rgb([255, 0, 0]);
#[cfg(feature = "nonogram")]
pub(crate) const GRAY_PIXEL: Rgb<u8> = Rgb([105, 105, 105]);

#[cfg(any(feature = "sudoku", feature = "nonogram"))]
pub(crate) const ROBOTO_MEDIUM: &[u8] = include_bytes!("../../resources/Roboto-Medium.ttf");

/// Converts a RgbBuffer to a Vec of bytes representing a PNG
pub fn image_to_png_bytes(image: &RgbBuffer) -> Result<Vec<u8>, ImageError> {
    let mut bytes = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;

    Ok(bytes)
}
