pub struct DisjointSet {
    elements: Vec<Element>,
}

#[derive(Clone)]
struct Element {
    root: bool,
    value: usize,
}

impl Element {
    fn new() -> Self {
        Element {
            root: true,
            value: 1,
        }
    }

    fn from_parent(parent: usize) -> Self {
        Element {
            root: false,
            value: parent,
        }
    }
}

impl DisjointSet {
    pub fn new() -> Self {
        DisjointSet {
            elements: Vec::new(),
        }
    }

    pub fn with_size(size: usize) -> Self {
        DisjointSet {
            elements: vec![Element::new(); size],
        }
    }

    pub fn add(&mut self) {
        self.elements.push(Element::new());
    }

    pub fn find(&mut self, idx: usize) -> Option<usize> {
        if idx >= self.elements.len() {
            return None;
        }

        Some(self.find_helper(idx))
    }

    pub fn common_set(&mut self, idx_one: usize, idx_two: usize) -> Option<bool> {
        Some(self.find(idx_one)? == self.find(idx_two)?)
    }

    pub fn union(&mut self, idx_one: usize, idx_two: usize) -> Option<usize> {
        let root_one = self.find(idx_one)?;
        let root_two = self.find(idx_two)?;

        if root_one == root_two {
            return Some(root_one);
        }

        let (smaller_idx, larger_idx) =
            if self.elements[root_one].value < self.elements[root_two].value {
                (root_one, root_two)
            } else {
                (root_two, root_one)
            };
        self.elements[larger_idx].value += self.elements[smaller_idx].value;
        self.elements[smaller_idx] = Element::from_parent(larger_idx);
        Some(smaller_idx)
    }

    fn find_helper(&mut self, idx: usize) -> usize {
        if self.elements[idx].root {
            return idx;
        }

        let root = self.find_helper(self.elements[idx].value);
        self.elements[idx].value = root;
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_size() {
        let set = DisjointSet::with_size(10);
        assert_eq!(set.elements.len(), 10);
        for elem in set.elements {
            assert!(elem.root);
            assert_eq!(elem.value, 1);
        }
    }

    #[test]
    fn add() {
        let mut set = DisjointSet::new();
        set.add();
        assert_eq!(set.elements.len(), 1);
        assert!(set.elements[0].root);
        assert_eq!(set.elements[0].value, 1);
    }

    #[test]
    fn union() {
        let mut set = DisjointSet::with_size(2);
        set.union(0, 1);
        assert!(set.elements[0].root);
        assert_eq!(set.elements[0].value, 2);
        assert!(!set.elements[1].root);
        assert_eq!(set.elements[1].value, 0);
    }

    #[test]
    fn union_by_size() {
        let mut set = DisjointSet::with_size(5);

        set.union(0, 1);
        set.union(2, 1);
        assert!(set.elements[0].root);
        assert_eq!(set.elements[0].value, 3);
        assert!(!set.elements[1].root);
        assert_eq!(set.elements[1].value, 0);
        assert!(!set.elements[2].root);
        assert_eq!(set.elements[2].value, 0);

        set.union(3, 4);
        set.union(4, 2);
        assert!(set.elements[0].root);
        assert_eq!(set.elements[0].value, 5);
        assert!(!set.elements[1].root);
        assert_eq!(set.elements[1].value, 0);
        assert!(!set.elements[2].root);
        assert_eq!(set.elements[2].value, 0);
        assert!(!set.elements[3].root);
        assert_eq!(set.elements[3].value, 0);
        assert!(!set.elements[4].root);
        assert_eq!(set.elements[4].value, 3);
    }

    #[test]
    fn find() {
        let mut set = DisjointSet::with_size(8);

        set.union(0, 1);
        set.union(1, 2);
        set.union(2, 3);

        set.union(4, 5);
        set.union(5, 6);

        assert_eq!(set.find(0).expect("should be some"), 0);
        assert_eq!(set.find(1).expect("should be some"), 0);
        assert_eq!(set.find(2).expect("should be some"), 0);
        assert_eq!(set.find(3).expect("should be some"), 0);

        assert_eq!(set.find(4).expect("should be some"), 4);
        assert_eq!(set.find(5).expect("should be some"), 4);
        assert_eq!(set.find(6).expect("should be some"), 4);

        assert_eq!(set.find(7).expect("should be some"), 7);

        assert!(set.find(8).is_none());
    }

    #[test]
    fn find_path_compression() {
        let mut set = DisjointSet::with_size(5);

        set.union(0, 1);
        set.union(2, 1);
        set.union(3, 4);
        set.union(4, 2);
        set.find(4);

        assert_eq!(set.elements[4].value, 0);
    }

    #[test]
    fn common_set() {
        let mut set = DisjointSet::with_size(8);

        set.union(0, 1);
        set.union(1, 2);
        set.union(2, 3);

        set.union(4, 5);
        set.union(5, 6);

        for i in 0..8 {
            for j in 0..8 {
                let common = set.common_set(i, j).expect("should be some");
                let expected = i == j
                    || (0..=3).contains(&i) && (0..=3).contains(&j)
                    || (4..=6).contains(&i) && (4..=6).contains(&j);

                assert_eq!(common, expected);
            }
        }

        assert!(set.common_set(8, 0).is_none());
        assert!(set.common_set(0, 8).is_none());
        assert!(set.common_set(8, 8).is_none());
    }
}
