//! B-tree insertion and split logic.

use crate::node::BNode;

/// Result of a node split: (median_key, median_val, new_right_child).
pub struct SplitResult<K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    pub mid_key: K,
    pub mid_val: V,
    pub right: Box<BNode<K, V>>,
}

/// Insert a key-value pair into the tree.
pub fn insert<K: Ord + Clone, V: Clone>(
    root: &mut Option<Box<BNode<K, V>>>,
    order: usize,
    key: K,
    value: V,
) {
    if root.is_none() {
        let mut leaf = BNode::new_leaf();
        leaf.insert_into_leaf(key, value);
        *root = Some(Box::new(leaf));
        return;
    }

    let root_node = root.as_mut().unwrap();
    let new_right = insert_recursive(root_node, order, key, value);

    if let Some(split) = new_right {
        let old_root = std::mem::replace(root_node, Box::new(BNode::new_internal()));
        root_node.keys.push(split.mid_key);
        root_node.values.push(split.mid_val);
        root_node.children.push(old_root);
        root_node.children.push(split.right);
    }
}

/// Returns Some(Box) with (median_key, median_val, new_right_child) if node split.
fn insert_recursive<K: Ord + Clone, V: Clone>(
    node: &mut BNode<K, V>,
    order: usize,
    key: K,
    value: V,
) -> Option<SplitResult<K, V>> {
    let max_keys = 2 * order - 1;

    if node.leaf {
        let idx = node.find_key_index(&key);
        if idx < node.keys.len() && node.keys[idx] == key {
            node.values[idx] = value;
            return None;
        }
        node.keys.insert(idx, key);
        node.values.insert(idx, value);

        if node.keys.len() > max_keys {
            split_node(node)
        } else {
            None
        }
    } else {
        let idx = node.find_key_index(&key);
        let child_split = insert_recursive(&mut node.children[idx], order, key, value);

        if let Some(split) = child_split {
            node.keys.insert(idx, split.mid_key);
            node.values.insert(idx, split.mid_val);
            node.children.insert(idx + 1, split.right);

            if node.keys.len() > max_keys {
                split_node(node)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn split_node<K: Ord + Clone, V: Clone>(
    node: &mut BNode<K, V>,
) -> Option<SplitResult<K, V>> {
    let mid = node.keys.len() / 2;
    let mid_key = node.keys.remove(mid);
    let mid_val = node.values.remove(mid);

    let is_leaf = node.leaf;

    let right = if is_leaf {
        let mut r = BNode::new_leaf();
        r.keys = node.keys.split_off(mid);
        r.values = node.values.split_off(mid);
        r
    } else {
        let mut r = BNode::new_internal();
        r.keys = node.keys.split_off(mid);
        r.values = node.values.split_off(mid);
        r.children = node.children.split_off(mid + 1);
        r
    };

    Some(SplitResult {
        mid_key,
        mid_val,
        right: Box::new(right),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::BTree;

    #[test]
    fn test_insert_single() {
        let mut root = None;
        insert(&mut root, 2, 10, "a");
        assert!(root.is_some());
        assert_eq!(root.as_ref().unwrap().keys, vec![10]);
    }

    #[test]
    fn test_insert_multiple_sorted() {
        let mut root = None;
        for i in 0..5 {
            insert(&mut root, 2, i, i * 10);
        }
        let all = root.as_ref().unwrap().collect_in_order();
        assert_eq!(all.len(), 5);
        for i in 0..5 {
            assert_eq!(all[i], (i, i * 10));
        }
    }

    #[test]
    fn test_insert_reverse() {
        let mut root = None;
        for i in (0..10).rev() {
            insert(&mut root, 2, i, i);
        }
        let all = root.as_ref().unwrap().collect_in_order();
        for i in 0..10 {
            assert_eq!(all[i].0, i);
        }
    }

    #[test]
    fn test_insert_causes_split() {
        let mut tree = BTree::new(2);
        for i in 0..10 {
            tree.insert(i, i * 100);
        }
        assert_eq!(tree.len(), 10);
        for i in 0..10 {
            assert_eq!(tree.search(&i), Some(&(i * 100)));
        }
    }

    #[test]
    fn test_insert_overwrite() {
        let mut tree = BTree::new(3);
        tree.insert(1, "old");
        tree.insert(1, "new");
        assert_eq!(tree.search(&1), Some(&"new"));
        assert_eq!(tree.len(), 1);
    }
}
