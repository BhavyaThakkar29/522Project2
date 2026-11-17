//! The AVL tree.
//!
//! An AVL tree is a binary search tree where, for every node, the heights of
//! the left and right subtrees differ by at most 1. After every insert or
//! delete the tree is checked on the way back up and fixed with rotations, so
//! its height always stays about log2(n) and every operation is O(log n).
//!
//! Insert and delete are written as recursive functions that hand back the
//! (possibly new) top node of the subtree they worked on. The caller then
//! hooks that node in with `set_left` / `set_right`, which also fixes the
//! parent pointer.

use crate::node::{self, Link, Node, NodeRef};
use crate::tree::SearchTree;
use std::cmp::Ordering;
use std::fmt::Display;
use std::rc::Weak;

pub struct AvlTree<T> {
    root: Link<T>,
}

impl<T> AvlTree<T> {
    pub fn new() -> Self {
        AvlTree { root: None }
    }

    /// Install a new root. The root has no parent.
    fn set_root(&mut self, root: Link<T>) {
        if let Some(r) = &root {
            r.borrow_mut().parent = Weak::new();
        }
        self.root = root;
    }
}

impl<T> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------- helpers

/// Height of a subtree; an empty one counts as 0.
fn height_of<T>(link: &Link<T>) -> usize {
    link.as_ref().map_or(0, |n| n.borrow().height)
}

/// Recompute a node's height from its two children.
fn update_height<T>(n: &NodeRef<T>) {
    let new_height = {
        let b = n.borrow();
        1 + height_of(&b.left).max(height_of(&b.right))
    };
    n.borrow_mut().height = new_height;
}

/// Left height minus right height. AVL requires this to be -1, 0 or 1.
fn balance_factor<T>(n: &NodeRef<T>) -> i32 {
    let b = n.borrow();
    height_of(&b.left) as i32 - height_of(&b.right) as i32
}

/// Rotate right around `y`: its left child `x` moves up and `y` becomes
/// `x`'s right child.
///
/// ```text
///      y            x
///     / \          / \
///    x   c   =>   a   y
///   / \              / \
///  a   b            b   c
/// ```
/// Returns `x`, the new top. The caller must hook `x` into the tree above.
fn rotate_right<T>(y: NodeRef<T>) -> NodeRef<T> {
    let x = y
        .borrow_mut()
        .left
        .take()
        .expect("rotate_right is only called on a node with a left child");
    let b = x.borrow_mut().right.take();
    node::set_left(&y, b);
    update_height(&y);
    node::set_right(&x, Some(y));
    update_height(&x);
    x
}

/// Mirror image of `rotate_right`.
fn rotate_left<T>(x: NodeRef<T>) -> NodeRef<T> {
    let y = x
        .borrow_mut()
        .right
        .take()
        .expect("rotate_left is only called on a node with a right child");
    let b = y.borrow_mut().left.take();
    node::set_right(&x, b);
    update_height(&x);
    node::set_left(&y, Some(x));
    update_height(&y);
    y
}

/// Fix `n` if its subtrees differ in height by 2. Returns the top node of
/// the (now balanced) subtree.
fn rebalance<T>(n: NodeRef<T>) -> NodeRef<T> {
    update_height(&n);
    let balance = balance_factor(&n);

    if balance > 1 {
        // Left side too tall.
        let left = n.borrow().left.clone().expect("left side is the tall one");
        if balance_factor(&left) < 0 {
            // Left-right case: first turn it into a left-left case.
            let new_left = rotate_left(left);
            node::set_left(&n, Some(new_left));
        }
        return rotate_right(n);
    }

    if balance < -1 {
        // Right side too tall.
        let right = n.borrow().right.clone().expect("right side is the tall one");
        if balance_factor(&right) > 0 {
            // Right-left case.
            let new_right = rotate_right(right);
            node::set_right(&n, Some(new_right));
        }
        return rotate_left(n);
    }

    n
}

// ----------------------------------------------------------- insert/delete

fn insert_node<T: Ord + Copy>(link: Link<T>, key: T, inserted: &mut bool) -> NodeRef<T> {
    let Some(n) = link else {
        *inserted = true;
        return Node::new_ref(key);
    };

    let order = key.cmp(&n.borrow().key);
    match order {
        Ordering::Equal => return n, // duplicate: add nothing
        Ordering::Less => {
            let left = n.borrow_mut().left.take();
            let new_left = insert_node(left, key, inserted);
            node::set_left(&n, Some(new_left));
        }
        Ordering::Greater => {
            let right = n.borrow_mut().right.take();
            let new_right = insert_node(right, key, inserted);
            node::set_right(&n, Some(new_right));
        }
    }

    if *inserted {
        rebalance(n)
    } else {
        n
    }
}

fn delete_node<T: Ord + Copy>(link: Link<T>, key: &T, removed: &mut bool) -> Link<T> {
    let n = link?; // empty subtree: key not found

    let order = key.cmp(&n.borrow().key);
    match order {
        Ordering::Less => {
            let left = n.borrow_mut().left.take();
            let new_left = delete_node(left, key, removed);
            node::set_left(&n, new_left);
        }
        Ordering::Greater => {
            let right = n.borrow_mut().right.take();
            let new_right = delete_node(right, key, removed);
            node::set_right(&n, new_right);
        }
        Ordering::Equal => {
            *removed = true;
            let left = n.borrow_mut().left.take();
            let right = n.borrow_mut().right.take();
            match (left, right) {
                (None, None) => return None,
                // One child: that child takes this node's place.
                (Some(only), None) | (None, Some(only)) => return Some(only),
                // Two children: copy in the next larger key (the smallest key
                // on the right), then delete that key from the right side.
                (Some(l), Some(r)) => {
                    let successor = node::min_key(&r);
                    n.borrow_mut().key = successor;
                    let mut ignored = false;
                    let new_right = delete_node(Some(r), &successor, &mut ignored);
                    node::set_left(&n, Some(l));
                    node::set_right(&n, new_right);
                }
            }
        }
    }

    Some(rebalance(n))
}

// ------------------------------------------------------------- the trait

impl<T: Ord + Copy + Display> SearchTree<T> for AvlTree<T> {
    fn insert(&mut self, key: T) -> bool {
        let mut inserted = false;
        let root = insert_node(self.root.take(), key, &mut inserted);
        self.set_root(Some(root));
        inserted
    }

    fn delete(&mut self, key: &T) -> bool {
        let mut removed = false;
        let root = delete_node(self.root.take(), key, &mut removed);
        self.set_root(root);
        removed
    }

    fn contains(&self, key: &T) -> bool {
        node::contains(&self.root, key)
    }

    fn count_leaves(&self) -> usize {
        node::count_leaves(&self.root)
    }

    fn height(&self) -> usize {
        height_of(&self.root)
    }

    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    fn in_order(&self) -> Vec<T> {
        node::in_order(&self.root)
    }

    fn render(&self) -> String {
        node::render(&self.root, &|n| format!("{} (h={})", n.key, n.height))
    }
}

// ----------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeSet;
    use std::rc::Rc;

    /// Check every AVL rule for the subtree at `link`. Returns its height,
    /// or `None` if anything is wrong:
    /// * keys are in binary-search-tree order (between `lo` and `hi`)
    /// * each parent pointer points at the real parent
    /// * the stored height is correct
    /// * the two subtrees differ in height by at most 1
    fn check<T: Ord + Copy>(
        link: &Link<T>,
        parent: &Weak<RefCell<Node<T>>>,
        lo: Option<T>,
        hi: Option<T>,
    ) -> Option<usize> {
        let Some(n) = link else { return Some(0) };
        let b = n.borrow();
        if lo.is_some_and(|lo| b.key <= lo) || hi.is_some_and(|hi| b.key >= hi) {
            return None;
        }
        if !Weak::ptr_eq(&b.parent, parent) {
            return None;
        }
        let me = Rc::downgrade(n);
        let left = check(&b.left, &me, lo, Some(b.key))?;
        let right = check(&b.right, &me, Some(b.key), hi)?;
        if left.abs_diff(right) > 1 || b.height != 1 + left.max(right) {
            return None;
        }
        Some(b.height)
    }

    fn assert_valid(tree: &AvlTree<i32>) {
        assert!(
            check(&tree.root, &Weak::new(), None, None).is_some(),
            "AVL rules broken:\n{}",
            tree.render()
        );
    }

    /// Small repeatable pseudo-random numbers (no extra crate needed).
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self, below: u64) -> i32 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((self.0 >> 33) % below) as i32
        }
    }

    fn tree_of(keys: &[i32]) -> AvlTree<i32> {
        let mut tree = AvlTree::new();
        for &k in keys {
            tree.insert(k);
        }
        tree
    }

    #[test]
    fn empty_tree() {
        let tree: AvlTree<i32> = AvlTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.count_leaves(), 0);
        assert_eq!(tree.in_order(), Vec::<i32>::new());
        assert_eq!(tree.render(), "(empty tree)\n");
        assert!(!tree.contains(&1));
    }

    #[test]
    fn duplicates_add_nothing() {
        let mut tree = tree_of(&[5, 3, 8]);
        assert!(!tree.insert(5));
        assert!(!tree.insert(3));
        assert_eq!(tree.in_order(), vec![3, 5, 8]);
        assert_valid(&tree);
    }

    #[test]
    fn ascending_inserts_stay_balanced() {
        // A plain binary search tree would become a 1000-long chain here.
        let mut tree = AvlTree::new();
        for k in 1..=1000 {
            tree.insert(k);
        }
        assert_valid(&tree);
        assert_eq!(tree.in_order(), (1..=1000).collect::<Vec<_>>());
        assert!(tree.height() <= 14, "height was {}", tree.height());
    }

    #[test]
    fn descending_inserts_stay_balanced() {
        let mut tree = AvlTree::new();
        for k in (1..=1000).rev() {
            tree.insert(k);
        }
        assert_valid(&tree);
        assert!(tree.height() <= 14);
    }

    #[test]
    fn all_four_rotation_cases() {
        // left-left, right-right, left-right, right-left
        for keys in [[3, 2, 1], [1, 2, 3], [3, 1, 2], [1, 3, 2]] {
            let tree = tree_of(&keys);
            assert_valid(&tree);
            assert_eq!(tree.height(), 2);
            assert_eq!(tree.in_order(), vec![1, 2, 3]);
        }
    }

    #[test]
    fn height_and_leaves() {
        // The AVL tree from the project description.
        let tree = tree_of(&[12, 8, 18, 5, 11, 17, 4]);
        assert_valid(&tree);
        assert_eq!(tree.in_order(), vec![4, 5, 8, 11, 12, 17, 18]);
        assert_eq!(tree.height(), node::height(&tree.root));
        assert_eq!(tree.count_leaves(), 3); // 4, 11 and 17
    }

    #[test]
    fn contains_finds_only_present_keys() {
        let tree = tree_of(&[10, 20, 30, 40, 50]);
        for k in [10, 20, 30, 40, 50] {
            assert!(tree.contains(&k));
        }
        for k in [0, 15, 55] {
            assert!(!tree.contains(&k));
        }
    }

    #[test]
    fn delete_leaf_one_child_two_children_and_root() {
        let mut tree = tree_of(&[50, 30, 70, 20, 40, 60, 80, 10]);
        assert!(tree.delete(&10)); // leaf
        assert_valid(&tree);
        assert!(tree.delete(&20)); // now a leaf after rebalancing, or one child
        assert_valid(&tree);
        assert!(tree.delete(&30)); // node with children
        assert_valid(&tree);
        assert!(tree.delete(&50)); // the root, two children
        assert_valid(&tree);
        assert_eq!(tree.in_order(), vec![40, 60, 70, 80]);
        assert!(!tree.delete(&999)); // not in the tree
        assert_eq!(tree.in_order(), vec![40, 60, 70, 80]);
    }

    #[test]
    fn delete_everything() {
        let mut tree = tree_of(&(1..=200).collect::<Vec<_>>());
        for k in 1..=200 {
            assert!(tree.delete(&k));
            assert_valid(&tree);
        }
        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
    }

    #[test]
    fn random_operations_match_a_standard_set() {
        let mut rng = Lcg(42);
        let mut tree = AvlTree::new();
        let mut reference = BTreeSet::new();
        for step in 0..6000 {
            let key = rng.next(300);
            if rng.next(3) == 0 {
                assert_eq!(tree.delete(&key), reference.remove(&key), "step {step}");
            } else {
                assert_eq!(tree.insert(key), reference.insert(key), "step {step}");
            }
            assert_valid(&tree);
            assert_eq!(tree.in_order(), reference.iter().copied().collect::<Vec<_>>());
            assert_eq!(tree.is_empty(), reference.is_empty());
        }
    }

    #[test]
    fn no_reference_cycles_so_memory_is_freed() {
        let tree = tree_of(&(1..=100).collect::<Vec<_>>());
        let root = Rc::downgrade(tree.root.as_ref().unwrap());
        assert!(root.upgrade().is_some());
        drop(tree);
        assert!(root.upgrade().is_none(), "the nodes were not freed");
    }

    #[test]
    fn render_shows_structure() {
        let tree = tree_of(&[2, 1, 3]);
        let expected = "2 (h=2)\n├── L: 1 (h=1)\n└── R: 3 (h=1)\n";
        assert_eq!(tree.render(), expected);
    }
}
