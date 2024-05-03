use rand::{seq::IteratorRandom, thread_rng};

pub fn choose_random<T>(vec: &mut Vec<T>) -> Option<T> {
    let idx = (0..vec.len()).choose(&mut thread_rng())?;
    Some(vec.swap_remove(idx))
}
