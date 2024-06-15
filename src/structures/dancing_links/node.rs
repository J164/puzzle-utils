use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::{null_mut, read},
};

pub struct Node {
    header: *mut Node,
    left: *mut Node,
    right: *mut Node,
    up: *mut Node,
    down: *mut Node,
    row: usize,
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
        row: usize,
    ) -> *mut Self {
        let ptr = unsafe { alloc(NODE_LAYOUT) } as *mut Node;
        let mut node = Node {
            header,
            left: ptr,
            right: ptr,
            up: ptr,
            down: ptr,
            row,
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
    pub unsafe fn new_header(previous: *mut Node, num_rows: usize) -> *mut Self {
        let ptr = unsafe { alloc(NODE_LAYOUT) } as *mut Node;
        let mut node = Node {
            header: null_mut(),
            left: ptr,
            right: ptr,
            up: ptr,
            down: ptr,
            row: num_rows,
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
    pub unsafe fn header(node: *mut Node) -> *mut Node {
        unsafe { (*node).header }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn right(node: *mut Node) -> *mut Node {
        unsafe { (*node).right }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn row(node: *mut Node) -> usize {
        unsafe { (*node).row }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn cover_column(mut node: *mut Node) {
        if unsafe { !(*node).header.is_null() } {
            node = unsafe { (*node).header };
        }

        unsafe { Node::cover_horizontal(node) };

        for row in unsafe { Node::iter_down(node).skip(1) } {
            for col in unsafe { Node::iter_right(row).skip(1) } {
                unsafe { Node::cover_vertical(col) };
                unsafe { (*(*col).header).row -= 1 };
            }
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn uncover_column(mut node: *mut Node) {
        if unsafe { !(*node).header.is_null() } {
            node = unsafe { (*node).header };
        }

        for row in unsafe { Node::iter_up(node).skip(1) } {
            for col in unsafe { Node::iter_left(row).skip(1) } {
                unsafe { (*(*col).header).row += 1 };
                unsafe { Node::uncover_vertical(col) };
            }
        }

        unsafe { Node::uncover_horizontal(node) };
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    pub unsafe fn free_chain(mut node: *mut Node) {
        if unsafe { !(*node).header.is_null() } {
            node = unsafe { (*node).header };
        }

        for row in unsafe { Node::iter_down(node).skip(1) } {
            for col in unsafe { Node::iter_right(row).skip(1) } {
                unsafe { dealloc(col as *mut u8, NODE_LAYOUT) };
            }

            unsafe { dealloc(row as *mut u8, NODE_LAYOUT) }
        }

        unsafe { dealloc(node as *mut u8, NODE_LAYOUT) };
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn cover_horizontal(node: *mut Node) {
        let Node { left, right, .. } = unsafe { read(node) };

        unsafe {
            (*left).right = right;
            (*right).left = left;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn cover_vertical(node: *mut Node) {
        let Node { up, down, .. } = unsafe { read(node) };

        unsafe {
            (*up).down = down;
            (*down).up = up;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn uncover_horizontal(node: *mut Node) {
        let Node { left, right, .. } = unsafe { read(node) };

        unsafe {
            (*left).right = node;
            (*right).left = node;
        }
    }

    /// # Safety
    ///
    /// 'node' must be a valid non-null pointer to a Node
    unsafe fn uncover_vertical(node: *mut Node) {
        let Node { up, down, .. } = unsafe { read(node) };

        unsafe {
            (*up).down = node;
            (*down).up = node;
        }
    }
}

macro_rules! node_iterator {
    ($direction:ident, $struct_name:ident, $fn_name:ident) => {
        struct $struct_name {
            original: *mut Node,
            current: *mut Node,
        }

        impl Iterator for $struct_name {
            type Item = *mut Node;

            fn next(&mut self) -> Option<Self::Item> {
                if self.current.is_null() {
                    return None;
                }

                let node = self.current;
                let next = unsafe { (*node).$direction };

                self.current = if next == self.original {
                    null_mut()
                } else {
                    next
                };

                Some(node)
            }
        }

        impl Node {
            /// # Safety
            ///
            /// 'node' must be a valid non-null pointer to a Node
            pub unsafe fn $fn_name(node: *mut Node) -> impl Iterator<Item = *mut Node> {
                $struct_name {
                    original: node,
                    current: node,
                }
            }
        }
    };
}

node_iterator!(left, LeftNodeIterator, iter_left);
node_iterator!(right, RightNodeIterator, iter_right);
node_iterator!(up, UpNodeIterator, iter_up);
node_iterator!(down, DownNodeIterator, iter_down);
