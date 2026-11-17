//! Pieces shared by every tree in this crate (the AVL tree now, the
//! red-black tree next), so none of this code is written twice.
//!
//! * `Node` is the one node type. The AVL tree uses `height`, the red-black
//!   tree will use `color`.
//! * The parent pointer is a `Weak` pointer. If it were an `Rc`, a parent and
//!   its child would point at each other, their counts would never reach zero
//!   and the memory would never be freed (a reference cycle).
//! * Counting leaves, searching, listing keys in order and drawing the tree
//!   work the same for any binary search tree, so they live here.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::Display;
use std::rc::{Rc, Weak};

/// A shared, changeable pointer to a node.
pub type NodeRef<T> = Rc<RefCell<Node<T>>>;

/// A child (or root) slot: either a node or empty.
pub type Link<T> = Option<NodeRef<T>>;

#[derive(Clone, Debug, PartialEq)]
pub enum NodeColor {
    Red,
    Black,
}

pub struct Node<T> {
    pub color: NodeColor,
    pub key: T,
    /// Height of the subtree that starts at this node (a lone node has height 1).
    pub height: usize,
    /// `Weak`, so parent and child do not keep each other alive.
    /// An empty `Weak` (from `Weak::new()`) means "no parent" (the root).
    pub parent: Weak<RefCell<Node<T>>>,
    pub left: Link<T>,
    pub right: Link<T>,
}

impl<T> Node<T> {
    /// A new node: red (as the red-black rules say), height 1, no parent, no children.
    pub fn new_ref(key: T) -> NodeRef<T> {
        Rc::new(RefCell::new(Node {
            color: NodeColor::Red,
            key,
            height: 1,
            parent: Weak::new(),
            left: None,
            right: None,
        }))
    }
}

/// Make `child` the left child of `parent` and point the child's parent
/// pointer back at `parent`.
pub fn set_left<T>(parent: &NodeRef<T>, child: Link<T>) {
    if let Some(c) = &child {
        c.borrow_mut().parent = Rc::downgrade(parent);
    }
    parent.borrow_mut().left = child;
}

/// Same as `set_left`, for the right child.
pub fn set_right<T>(parent: &NodeRef<T>, child: Link<T>) {
    if let Some(c) = &child {
        c.borrow_mut().parent = Rc::downgrade(parent);
    }
    parent.borrow_mut().right = child;
}

/// Is `key` in the tree? Walks down from the root, going left or right.
pub fn contains<T: Ord>(root: &Link<T>, key: &T) -> bool {
    let mut current = root.clone();
    while let Some(node) = current {
        let next = {
            let n = node.borrow();
            match key.cmp(&n.key) {
                Ordering::Equal => return true,
                Ordering::Less => n.left.clone(),
                Ordering::Greater => n.right.clone(),
            }
        };
        current = next;
    }
    false
}

/// The smallest key in the subtree that starts at `root`.
pub fn min_key<T: Copy>(root: &NodeRef<T>) -> T {
    let mut current = Rc::clone(root);
    loop {
        let next = current.borrow().left.clone();
        match next {
            Some(left) => current = left,
            None => {
                let key = current.borrow().key;
                return key;
            }
        }
    }
}

/// Height of the tree, counted by walking it: an empty tree is 0, a single
/// node is 1. (The AVL tree keeps the height in each node instead.)
pub fn height<T>(root: &Link<T>) -> usize {
    match root {
        None => 0,
        Some(node) => {
            let n = node.borrow();
            1 + height(&n.left).max(height(&n.right))
        }
    }
}

/// Number of leaves: nodes that have no children.
pub fn count_leaves<T>(root: &Link<T>) -> usize {
    match root {
        None => 0,
        Some(node) => {
            let n = node.borrow();
            if n.left.is_none() && n.right.is_none() {
                1
            } else {
                count_leaves(&n.left) + count_leaves(&n.right)
            }
        }
    }
}

/// All keys, smallest to largest (left subtree, node, right subtree).
pub fn in_order<T: Copy>(root: &Link<T>) -> Vec<T> {
    fn walk<T: Copy>(link: &Link<T>, out: &mut Vec<T>) {
        if let Some(node) = link {
            let n = node.borrow();
            walk(&n.left, out);
            out.push(n.key);
            walk(&n.right, out);
        }
    }
    let mut keys = Vec::new();
    walk(root, &mut keys);
    keys
}

/// Draw the tree as text. `label` decides what is written for each node, so
/// the AVL tree can show heights and the red-black tree can show colors.
///
/// ```text
/// 12 (h=3)
/// ├── L: 8 (h=2)
/// │   └── L: 5 (h=1)
/// └── R: 18 (h=1)
/// ```
pub fn render<T: Display>(root: &Link<T>, label: &dyn Fn(&Node<T>) -> String) -> String {
    match root {
        None => "(empty tree)\n".to_string(),
        Some(node) => {
            let mut out = String::new();
            out += &label(&node.borrow());
            out.push('\n');
            let (left, right) = children(node);
            render_children(&left, &right, "", label, &mut out);
            out
        }
    }
}

fn children<T>(node: &NodeRef<T>) -> (Link<T>, Link<T>) {
    let n = node.borrow();
    (n.left.clone(), n.right.clone())
}

fn render_children<T: Display>(
    left: &Link<T>,
    right: &Link<T>,
    prefix: &str,
    label: &dyn Fn(&Node<T>) -> String,
    out: &mut String,
) {
    let kids: Vec<(&str, &NodeRef<T>)> = [("L", left), ("R", right)]
        .into_iter()
        .filter_map(|(side, link)| link.as_ref().map(|node| (side, node)))
        .collect();

    for (i, (side, node)) in kids.iter().enumerate() {
        let is_last = i + 1 == kids.len();
        let branch = if is_last { "└── " } else { "├── " };
        *out += &format!("{prefix}{branch}{side}: {}\n", label(&node.borrow()));

        let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
        let (l, r) = children(node);
        render_children(&l, &r, &child_prefix, label, out);
    }
}
