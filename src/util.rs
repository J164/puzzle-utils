use rand::{seq::IteratorRandom, thread_rng};

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
