//! Self-balancing binary search trees (ECE 522, Project 2).
//!
//! * `node`: the node type and the code every tree shares
//! * `tree`: the `SearchTree` trait (what a tree can do)
//! * `avl`: the AVL tree

pub mod avl;
pub mod node;
pub mod tree;

pub use avl::AvlTree;
pub use tree::SearchTree;
