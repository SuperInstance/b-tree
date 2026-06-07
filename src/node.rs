//! B-tree node structure and operations.

/// A node in the B-tree.
#[derive(Debug, Clone)]
pub struct BNode<K: Ord + Clone, V: Clone> {
    /// Is this a leaf node?
    pub leaf: bool,
    /// Sorted keys
    pub keys: Vec<K>,
    /// Values (parallel to keys for leaves)
    pub values: Vec<V>,
    /// Child pointers (for internal nodes, len = keys.len() + 1)
    pub children: Vec<Box<BNode<K, V>>>,
}

impl<K: Ord + Clone, V: Clone> BNode<K, V> {
    /// Create a new leaf node.
    pub fn new_leaf() -> Self {
        BNode {
            leaf: true,
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create a new internal node.
    pub fn new_internal() -> Self {
        BNode {
            leaf: false,
            keys: Vec::new(),
            values: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Number of keys in this node.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Find the position where a key would go (or already is).
    pub fn find_key_index(&self, key: &K) -> usize {
        match self.keys.binary_search(key) {
            Ok(i) => i,
            Err(i) => i,
        }
    }

    /// Check if the node contains the exact key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.keys.binary_search(key).is_ok()
    }

    /// Get the value at a key index.
    pub fn get_value(&self, index: usize) -> Option<&V> {
        self.values.get(index)
    }

    /// Get a mutable reference to the value at a key index.
    pub fn get_value_mut(&mut self, index: usize) -> Option<&mut V> {
        self.values.get_mut(index)
    }

    /// Get a child by index.
    pub fn get_child(&self, index: usize) -> Option<&BNode<K, V>> {
        self.children.get(index).map(|b| b.as_ref())
    }

    /// Insert a key-value pair into a leaf node at the correct position.
    pub fn insert_into_leaf(&mut self, key: K, value: V) {
        let idx = self.find_key_index(&key);
        if idx < self.keys.len() && self.keys[idx] == key {
            self.values[idx] = value;
        } else {
            self.keys.insert(idx, key);
            self.values.insert(idx, value);
        }
    }

    /// Check if this node is at or above minimum capacity for the given order.
    pub fn is_above_min(&self, min_keys: usize) -> bool {
        self.keys.len() > min_keys
    }

    /// Check if this node is at maximum capacity for the given order.
    pub fn is_full(&self, max_keys: usize) -> bool {
        self.keys.len() >= max_keys
    }

    /// Compute the depth of the subtree rooted at this node.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Collect all key-value pairs in sorted order.
    pub fn collect_in_order(&self) -> Vec<(K, V)> {
        let mut result = Vec::new();
        self.collect_in_order_into(&mut result);
        result
    }

    fn collect_in_order_into(&self, result: &mut Vec<(K, V)>) {
        if self.leaf {
            for i in 0..self.keys.len() {
                result.push((self.keys[i].clone(), self.values[i].clone()));
            }
        } else {
            for i in 0..self.keys.len() {
                self.children[i].collect_in_order_into(result);
                result.push((self.keys[i].clone(), self.values[i].clone()));
            }
            if !self.children.is_empty() {
                self.children.last().unwrap().collect_in_order_into(result);
            }
        }
    }

    /// Find the index of the minimum key in the subtree.
    pub fn find_min_index(&self) -> usize {
        let mut node = self;
        let mut idx = 0;
        while !node.leaf {
            node = &node.children[0];
            idx = 0;
        }
        idx
    }

    /// Get the minimum key in this subtree.
    pub fn min_key(&self) -> Option<&K> {
        if self.leaf {
            self.keys.first()
        } else {
            self.children.first().and_then(|c| c.min_key())
        }
    }

    /// Get the maximum key in this subtree.
    pub fn max_key(&self) -> Option<&K> {
        if self.leaf {
            self.keys.last()
        } else {
            self.children.last().and_then(|c| c.max_key())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_leaf() {
        let node: BNode<i32, i32> = BNode::new_leaf();
        assert!(node.leaf);
        assert!(node.keys.is_empty());
        assert!(node.values.is_empty());
    }

    #[test]
    fn test_new_internal() {
        let node: BNode<i32, i32> = BNode::new_internal();
        assert!(!node.leaf);
        assert!(node.keys.is_empty());
    }

    #[test]
    fn test_insert_into_leaf() {
        let mut node = BNode::new_leaf();
        node.insert_into_leaf(3, 30);
        node.insert_into_leaf(1, 10);
        node.insert_into_leaf(2, 20);
        assert_eq!(node.keys, vec![1, 2, 3]);
        assert_eq!(node.values, vec![10, 20, 30]);
    }

    #[test]
    fn test_insert_overwrite() {
        let mut node = BNode::new_leaf();
        node.insert_into_leaf(1, 10);
        node.insert_into_leaf(1, 99);
        assert_eq!(node.values, vec![99]);
        assert_eq!(node.key_count(), 1);
    }

    #[test]
    fn test_find_key_index() {
        let mut node: BNode<i32, i32> = BNode::new_leaf();
        node.keys = vec![1, 3, 5, 7];
        assert_eq!(node.find_key_index(&0), 0);
        assert_eq!(node.find_key_index(&1), 0);
        assert_eq!(node.find_key_index(&4), 2);
        assert_eq!(node.find_key_index(&7), 3);
        assert_eq!(node.find_key_index(&9), 4);
    }

    #[test]
    fn test_depth_leaf() {
        let node: BNode<i32, i32> = BNode::new_leaf();
        assert_eq!(node.depth(), 1);
    }

    #[test]
    fn test_collect_in_order() {
        let mut node: BNode<i32, i32> = BNode::new_leaf();
        node.insert_into_leaf(3, 30);
        node.insert_into_leaf(1, 10);
        node.insert_into_leaf(2, 20);
        assert_eq!(node.collect_in_order(), vec![(1, 10), (2, 20), (3, 30)]);
    }
}
