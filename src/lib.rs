//! B-tree implementation with insertion, deletion, search, and range queries.

pub mod delete;
pub mod insert;
pub mod node;
pub mod range;
pub mod tree;

pub use tree::BTree;
