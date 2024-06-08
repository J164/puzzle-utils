use std::{
    alloc::{alloc, Layout},
    ptr,
};

pub struct Node {
    pub header: *mut Node,
    pub left: *mut Node,
    pub right: *mut Node,
    pub up: *mut Node,
    pub down: *mut Node,
    pub is_header: bool,
}

pub const NODE_LAYOUT: Layout = Layout::new::<Node>();

impl Node {
    pub fn new(header: *mut Node, row_previous: *mut Node, col_previous: *mut Node) -> *mut Self {
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

    pub fn new_header(previous: *mut Node) -> *mut Self {
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
}
