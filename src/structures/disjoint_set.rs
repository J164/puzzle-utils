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
