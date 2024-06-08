use std::{
    alloc::{alloc, Layout},
    ptr::{self, null_mut},
};

pub struct Node {
    header: *mut Node,
    left: *mut Node,
    right: *mut Node,
    up: *mut Node,
    down: *mut Node,
    is_header: bool,
}

pub const NODE_LAYOUT: Layout = Layout::new::<Node>();

impl Node {
    /// # Safety
    ///
    /// 'header' must be a non-null valid pointer to the header Node in the same column as the new Node
    /// 'row_previous' must be a valid pointer (possibly null) to a Node in the row to place the new Node
    /// 'col_previous' must be a valid pointer (possibly null) to a Node in the column to place the new Node
    pub unsafe fn new(
        header: *mut Node,
        row_previous: *mut Node,
        col_previous: *mut Node,
    ) -> *mut Self {
        let ptr = unsafe { alloc(NODE_LAYOUT) } as *mut Node;
        let mut node = Node {
            header,
            left: ptr,
            right: ptr,
            up: ptr,
            down: ptr,
            is_header: false,
        };

        if !row_previous.is_null() {
            node.left = row_previous;
            node.right = unsafe { (*row_previous).right };

            unsafe {
                (*(*row_previous).right).left = ptr;
                (*row_previous).right = ptr;
            }
        }

        if !col_previous.is_null() {
            node.up = col_previous;
            node.down = unsafe { (*col_previous).down };

            unsafe {
                (*(*col_previous).down).up = ptr;
                (*col_previous).down = ptr;
            }
        }

        unsafe { *ptr = node }

        ptr
    }

    /// # Safety
    ///
    /// 'previous' must be a valid pointer (possibly null) to a header Node
    pub unsafe fn new_header(previous: *mut Node) -> *mut Self {
        let ptr = unsafe { alloc(NODE_LAYOUT) } as *mut Node;
        let mut node = Node {
            header: ptr::null_mut(),
            left: ptr,
            right: ptr,
            up: ptr,
            down: ptr,
            is_header: true,
        };

        if !previous.is_null() {
            node.left = previous;
            node.right = unsafe { (*previous).right };

            unsafe {
                (*(*previous).right).left = ptr;
                (*previous).right = ptr;
            }
        }

        unsafe { *ptr = node }

        ptr
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn cover_horizontal(node: *mut Node) {
        let Node { left, right, .. } = unsafe { ptr::read(node) };

        unsafe {
            (*left).right = right;
            (*right).left = left;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn cover_vertical(node: *mut Node) {
        let Node { up, down, .. } = unsafe { ptr::read(node) };

        unsafe {
            (*up).down = down;
            (*down).up = up;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn uncover_horizontal(node: *mut Node) {
        let Node { left, right, .. } = unsafe { ptr::read(node) };

        unsafe {
            (*left).right = node;
            (*right).left = node;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn uncover_vertical(node: *mut Node) {
        let Node { up, down, .. } = unsafe { ptr::read(node) };

        unsafe {
            (*up).down = node;
            (*down).up = node;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn iter_right(node: *mut Node) -> impl Iterator<Item = *mut Node> {
        RightNodeIterator {
            original: node,
            current: node,
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn iter_down(node: *mut Node) -> impl Iterator<Item = *mut Node> {
        DownNodeIterator {
            original: node,
            current: node,
        }
    }
}

struct RightNodeIterator {
    original: *mut Node,
    current: *mut Node,
}

impl Iterator for RightNodeIterator {
    type Item = *mut Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        let node = self.current;
        let next = unsafe { (*node).right };

        self.current = if next == self.original {
            null_mut()
        } else {
            next
        };
        
        Some(node)
    }
}

pub struct DownNodeIterator {
    original: *mut Node,
    current: *mut Node,
}

impl Iterator for DownNodeIterator {
    type Item = *mut Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        let node = self.current;
        let next = unsafe { (*node).down };

        self.current = if next == self.original {
            null_mut()
        } else {
            next
        };
        
        Some(node)
    }
}
