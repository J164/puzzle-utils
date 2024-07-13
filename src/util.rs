use image::Rgb;
use rand::{seq::IteratorRandom, thread_rng};

pub const WHITE_PIXEL: Rgb<u8> = Rgb([255, 255, 255]);
pub const BLACK_PIXEL: Rgb<u8> = Rgb([0, 0, 0]);
pub const RED_PIXEL: Rgb<u8> = Rgb([255, 0, 0]);
pub const GRAY_PIXEL: Rgb<u8> = Rgb([105, 105, 105]);

pub const ROBOTO_MEDIUM: &[u8] = include_bytes!("../resources/Roboto-Medium.ttf");

pub fn choose_random<T>(vec: &mut Vec<T>) -> Option<T> {
    let idx = (0..vec.len()).choose(&mut thread_rng())?;
    Some(vec.swap_remove(idx))
}

#[cfg(test)]
mod choose_random_tests {
    use super::*;

    #[test]
    fn removes_element_from_vec() {
        let mut vec = vec![1, 2, 3, 4, 5];
        let elem = choose_random(&mut vec).expect("element should be chosen");
        assert!(!vec.contains(&elem));
    }

    #[test]
    fn returns_none_if_empty() {
        let mut vec = Vec::<i32>::new();
        assert!(choose_random(&mut vec).is_none());
    }
}
