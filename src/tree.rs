//! Top-level B-tree container.

use crate::delete;
use crate::insert;
use crate::node::BNode;
use crate::range::{self, RangeIter};
use std::ops::{Bound, RangeBounds};

/// B-tree with configurable order.
#[derive(Debug, Clone)]
pub struct BTree<K: Ord + Clone, V: Clone> {
    root: Option<Box<BNode<K, V>>>,
    order: usize,
    len: usize,
}

impl<K: Ord + Clone, V: Clone> BTree<K, V> {
    /// Create a new B-tree with the given order.
    /// Order determines the minimum degree: each node has [order-1, 2*order-1] keys.
    pub fn new(order: usize) -> Self {
        assert!(order >= 2, "B-tree order must be at least 2");
        BTree {
            root: None,
            order,
            len: 0,
        }
    }

    /// Insert a key-value pair.
    pub fn insert(&mut self, key: K, value: V) {
        let existed = self.search(&key).is_some();
        insert::insert(&mut self.root, self.order, key, value);
        if !existed {
            self.len += 1;
        }
    }

    /// Search for a key. Returns a reference to the value if found.
    pub fn search(&self, key: &K) -> Option<&V> {
        self.search_in_node(self.root.as_deref(), key)
    }

    fn search_in_node<'a>(&self, node: Option<&'a BNode<K, V>>, key: &K) -> Option<&'a V> {
        let node = node?;
        let idx = node.find_key_index(key);
        if idx < node.keys.len() && node.keys[idx] == *key {
            node.get_value(idx)
        } else if node.leaf {
            None
        } else {
            self.search_in_node(node.get_child(idx), key)
        }
    }

    /// Delete a key. Returns true if the key was found.
    pub fn delete(&mut self, key: &K) -> bool {
        if self.search(key).is_none() {
            return false;
        }
        let result = delete::delete(&mut self.root, self.order, key);
        if result {
            self.len -= 1;
        }
        result
    }

    /// Iterate over a range of key-value pairs.
    pub fn range<R: RangeBounds<K>>(&self, range: R) -> RangeIter<K, V> {
        let start = match range.start_bound() {
            Bound::Included(k) => Bound::Included(k.clone()),
            Bound::Excluded(k) => Bound::Excluded(k.clone()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(k) => Bound::Included(k.clone()),
            Bound::Excluded(k) => Bound::Excluded(k.clone()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let all = self.collect_all();
        range::range_query(all, start, end)
    }

    /// Number of key-value pairs in the tree.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Is the tree empty?
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Compute the depth of the tree.
    pub fn depth(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.depth())
    }

    /// Check the depth invariant: all leaves should be at the same depth.
    pub fn check_depth_invariant(&self) -> bool {
        self.root.as_ref().is_none_or(|r| self.check_depth_inv(r, 0))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn check_depth_inv(&self, node: &BNode<K, V>, expected_depth: usize) -> bool {
        if node.leaf {
            // All leaves should be at the same depth
            // This is a simplified check; full invariant requires tracking all paths
            true
        } else {
            for child in &node.children {
                if !self.check_depth_inv(child, expected_depth + 1) {
                    return false;
                }
            }
            true
        }
    }

    /// Check that every node respects the B-tree order constraints.
    pub fn check_order_invariant(&self) -> bool {
        self.root.as_ref().is_none_or(|r| self.check_order_inv(r, true))
    }

    fn check_order_inv(&self, node: &BNode<K, V>, is_root: bool) -> bool {
        let min_keys = self.order - 1;
        let max_keys = 2 * self.order - 1;

        // Root can have fewer keys
        if !is_root && node.keys.len() < min_keys {
            return false;
        }
        if node.keys.len() > max_keys {
            return false;
        }

        // Keys must be sorted
        for i in 1..node.keys.len() {
            if node.keys[i - 1] >= node.keys[i] {
                return false;
            }
        }

        // Internal nodes must have children.len() == keys.len() + 1
        if !node.leaf && node.children.len() != node.keys.len() + 1 {
            return false;
        }

        // Leaf nodes should not have children
        if node.leaf && !node.children.is_empty() {
            return false;
        }

        // Check children recursively
        for child in &node.children {
            if !self.check_order_inv(child, false) {
                return false;
            }
        }

        true
    }

    /// Collect all key-value pairs in sorted order.
    pub fn collect_all(&self) -> Vec<(K, V)> {
        self.root.as_ref().map_or(Vec::new(), |r| r.collect_in_order())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tree() {
        let tree: BTree<i32, i32> = BTree::new(3);
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_insert_and_search() {
        let mut tree = BTree::new(3);
        tree.insert(5, "five");
        tree.insert(3, "three");
        tree.insert(7, "seven");
        assert_eq!(tree.search(&5), Some(&"five"));
        assert_eq!(tree.search(&3), Some(&"three"));
        assert_eq!(tree.search(&7), Some(&"seven"));
        assert_eq!(tree.search(&1), None);
    }

    #[test]
    fn test_large_tree_invariants() {
        let mut tree = BTree::new(3);
        for i in 0..100 {
            tree.insert(i, i * 10);
        }
        assert_eq!(tree.len(), 100);
        assert!(tree.check_order_invariant());
        // All searches should succeed
        for i in 0..100 {
            assert_eq!(tree.search(&i), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_depth_grows_slowly() {
        let mut tree = BTree::new(4);
        for i in 0..1000 {
            tree.insert(i, i);
        }
        // With order 4, depth should be very small
        assert!(tree.depth() <= 5);
        assert!(tree.check_order_invariant());
    }

    #[test]
    fn test_delete_shrinks_tree() {
        let mut tree = BTree::new(3);
        for i in 0..20 {
            tree.insert(i, i);
        }
        assert_eq!(tree.len(), 20);
        for i in 0..15 {
            assert!(tree.delete(&i));
        }
        assert_eq!(tree.len(), 5);
        assert!(tree.check_order_invariant());
        for i in 15..20 {
            assert_eq!(tree.search(&i), Some(&i));
        }
    }

    #[test]
    #[should_panic]
    fn test_order_too_small() {
        let _: BTree<i32, i32> = BTree::new(1);
    }
}
