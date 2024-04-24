use image::Rgb;
use rand::{seq::IteratorRandom, thread_rng};

pub const WHITE_PIXEL: Rgb<u8> = Rgb([255, 255, 255]);
pub const BLACK_PIXEL: Rgb<u8> = Rgb([0, 0, 0]);
pub const RED_PIXEL: Rgb<u8> = Rgb([255, 0, 0]);

pub fn choose_random<T>(vec: &mut Vec<T>) -> Option<T> {
    let idx = (0..vec.len()).choose(&mut thread_rng())?;
    Some(vec.swap_remove(idx))
}
