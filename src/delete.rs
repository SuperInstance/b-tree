//! B-tree deletion, rebalancing, and merge logic.

use crate::node::BNode;

/// Delete a key from the B-tree. Returns true if the key was found and removed.
pub fn delete<K: Ord + Clone, V: Clone>(
    root: &mut Option<Box<BNode<K, V>>>,
    order: usize,
    key: &K,
) -> bool {
    if root.is_none() {
        return false;
    }

    let result = delete_recursive(root.as_deref_mut(), order, key);

    // If root became empty internal node with one child, shrink tree
    if let Some(r) = root.as_deref_mut() {
        if !r.leaf && r.keys.is_empty() && r.children.len() == 1 {
            let only_child = r.children.pop().unwrap();
            *root = Some(only_child);
        }
    }

    result
}

fn delete_recursive<K: Ord + Clone, V: Clone>(
    node: Option<&mut BNode<K, V>>,
    order: usize,
    key: &K,
) -> bool {
    let node = match node {
        Some(n) => n,
        None => return false,
    };
    let min_keys = order - 1;
    let idx = node.find_key_index(key);

    if node.leaf {
        if idx < node.keys.len() && node.keys[idx] == *key {
            node.keys.remove(idx);
            node.values.remove(idx);
            true
        } else {
            false
        }
    } else if idx < node.keys.len() && node.keys[idx] == *key {
        delete_key_from_internal(node, order, idx, min_keys)
    } else {
        ensure_child_min(node, order, idx, min_keys);
        // Re-find the correct child index after rebalancing
        let new_idx = node.find_key_index(key);
        if new_idx < node.keys.len() && node.keys[new_idx] == *key {
            delete_key_from_internal(node, order, new_idx, min_keys)
        } else if new_idx < node.children.len() {
            delete_recursive(Some(&mut node.children[new_idx]), order, key)
        } else {
            false
        }
    }
}

fn delete_key_from_internal<K: Ord + Clone, V: Clone>(
    node: &mut BNode<K, V>,
    order: usize,
    idx: usize,
    min_keys: usize,
) -> bool {
    let child_key = node.children[idx].keys[0].clone();

    if node.children[idx].keys.len() > min_keys {
        let (pred_k, pred_v) = take_predecessor(&mut node.children[idx]);
        node.keys[idx] = pred_k;
        node.values[idx] = pred_v;
        true
    } else if node.children[idx + 1].keys.len() > min_keys {
        let (succ_k, succ_v) = take_successor(&mut node.children[idx + 1]);
        node.keys[idx] = succ_k;
        node.values[idx] = succ_v;
        true
    } else {
        let key = node.keys.remove(idx);
        let val = node.values.remove(idx);
        let right = node.children.remove(idx + 1);
        merge_into_left(&mut node.children[idx], key, val, right);
        delete_recursive(Some(&mut node.children[idx]), order, &child_key)
    }
}

fn take_predecessor<K: Ord + Clone, V: Clone>(node: &mut BNode<K, V>) -> (K, V) {
    if node.leaf {
        let last = node.keys.len() - 1;
        (node.keys.remove(last), node.values.remove(last))
    } else {
        take_predecessor(node.children.last_mut().unwrap())
    }
}

fn take_successor<K: Ord + Clone, V: Clone>(node: &mut BNode<K, V>) -> (K, V) {
    if node.leaf {
        (node.keys.remove(0), node.values.remove(0))
    } else {
        take_successor(node.children.first_mut().unwrap())
    }
}

fn merge_into_left<K: Ord + Clone, V: Clone>(
    left: &mut BNode<K, V>,
    key: K,
    value: V,
    #[allow(clippy::boxed_local)] right: Box<BNode<K, V>>,
) {
    left.keys.push(key);
    left.values.push(value);
    left.keys.extend(right.keys);
    left.values.extend(right.values);
    if !right.leaf {
        left.children.extend(right.children);
    }
}

fn ensure_child_min<K: Ord + Clone, V: Clone>(
    node: &mut BNode<K, V>,
    _order: usize,
    child_idx: usize,
    min_keys: usize,
) {
    if node.children[child_idx].keys.len() > min_keys {
        return;
    }

    if child_idx > 0 && node.children[child_idx - 1].keys.len() > min_keys {
        borrow_from_prev(node, child_idx);
    } else if child_idx + 1 < node.children.len()
        && node.children[child_idx + 1].keys.len() > min_keys
    {
        borrow_from_next(node, child_idx);
    } else {
        let merge_idx = if child_idx > 0 {
            child_idx - 1
        } else {
            child_idx
        };
        let key = node.keys.remove(merge_idx);
        let val = node.values.remove(merge_idx);
        let right = node.children.remove(merge_idx + 1);
        merge_into_left(&mut node.children[merge_idx], key, val, right);
    }
}

fn borrow_from_prev<K: Ord + Clone, V: Clone>(node: &mut BNode<K, V>, child_idx: usize) {
    let sep_idx = child_idx - 1;
    let sep_key = std::mem::replace(&mut node.keys[sep_idx], node.children[child_idx - 1].keys.last().unwrap().clone());
    let sep_val = std::mem::replace(&mut node.values[sep_idx], node.children[child_idx - 1].values.last().unwrap().clone());

    let prev = &mut node.children[child_idx - 1];
    let borrowed_key = prev.keys.pop().unwrap();
    let borrowed_val = prev.values.pop().unwrap();
    let borrowed_child = if !prev.leaf { prev.children.pop() } else { None };

    node.keys[sep_idx] = borrowed_key;
    node.values[sep_idx] = borrowed_val;

    let child = &mut node.children[child_idx];
    child.keys.insert(0, sep_key);
    child.values.insert(0, sep_val);
    if let Some(bc) = borrowed_child {
        child.children.insert(0, bc);
    }
}

fn borrow_from_next<K: Ord + Clone, V: Clone>(node: &mut BNode<K, V>, child_idx: usize) {
    let sep_key = std::mem::replace(&mut node.keys[child_idx], node.children[child_idx + 1].keys[0].clone());
    let sep_val = std::mem::replace(&mut node.values[child_idx], node.children[child_idx + 1].values[0].clone());

    let next = &mut node.children[child_idx + 1];
    let borrowed_key = next.keys.remove(0);
    let borrowed_val = next.values.remove(0);
    let borrowed_child = if !next.leaf { Some(next.children.remove(0)) } else { None };

    node.keys[child_idx] = borrowed_key;
    node.values[child_idx] = borrowed_val;

    let child = &mut node.children[child_idx];
    child.keys.push(sep_key);
    child.values.push(sep_val);
    if let Some(bc) = borrowed_child {
        child.children.push(bc);
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::BTree;

    #[test]
    fn test_delete_from_leaf() {
        let mut tree = BTree::new(3);
        tree.insert(1, "a");
        tree.insert(2, "b");
        tree.insert(3, "c");
        assert!(tree.delete(&2));
        assert_eq!(tree.search(&2), None);
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut tree = BTree::new(3);
        tree.insert(1, "a");
        assert!(!tree.delete(&99));
    }

    #[test]
    fn test_delete_all() {
        let mut tree = BTree::new(3);
        for i in 0..30 {
            tree.insert(i, i);
        }
        for i in 0..30 {
            assert!(tree.delete(&i), "Failed to delete key {}", i);
        }
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_delete_rebalance() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i * 10);
        }
        for i in 3..7 {
            assert!(tree.delete(&i));
        }
        assert_eq!(tree.len(), 6);
        for i in 0..10 {
            if (3..7).contains(&i) {
                assert_eq!(tree.search(&i), None);
            } else {
                assert_eq!(tree.search(&i), Some(&(i * 10)));
            }
        }
    }

    #[test]
    fn test_delete_then_insert() {
        let mut tree = BTree::new(3);
        tree.insert(1, "a");
        tree.insert(2, "b");
        tree.delete(&1);
        tree.insert(1, "new_a");
        assert_eq!(tree.search(&1), Some(&"new_a"));
    }
}
