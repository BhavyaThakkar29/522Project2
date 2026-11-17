//! The list of things every tree in this crate can do.
//!
//! The AVL tree implements this trait, and the red-black tree will too. The
//! command-line program and the benchmarks only talk to this trait, so they
//! work with either tree and are written once.

use std::fmt::Display;

pub trait SearchTree<T: Display> {
    /// Add `key`. Returns `false` (and changes nothing) if it is already there.
    fn insert(&mut self, key: T) -> bool;

    /// Remove `key`. Returns `false` if it was not in the tree.
    fn delete(&mut self, key: &T) -> bool;

    fn contains(&self, key: &T) -> bool;

    /// Number of nodes that have no children.
    fn count_leaves(&self) -> usize;

    /// Empty tree = 0, a single node = 1.
    fn height(&self) -> usize;

    fn is_empty(&self) -> bool;

    /// All keys, smallest to largest.
    fn in_order(&self) -> Vec<T>;

    /// The tree drawn as text.
    fn render(&self) -> String;

    fn print_in_order(&self) {
        let keys: Vec<String> = self.in_order().iter().map(|k| k.to_string()).collect();
        if keys.is_empty() {
            println!("(empty tree)");
        } else {
            println!("{}", keys.join(" "));
        }
    }

    fn print_tree(&self) {
        print!("{}", self.render());
    }
}
