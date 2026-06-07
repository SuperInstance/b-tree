//! Range query iteration over B-tree keys.

use std::ops::Bound;

/// Iterator over a range of key-value pairs in the B-tree.
/// Uses a simple approach: collect all pairs in order, then filter by range.
pub struct RangeIter<K: Ord + Clone, V: Clone> {
    items: Vec<(K, V)>,
    index: usize,
}

impl<K: Ord + Clone, V: Clone> RangeIter<K, V> {
    /// Create a new range iterator.
    pub fn new(all_items: Vec<(K, V)>, start: Bound<K>, end: Bound<K>) -> Self {
        let mut filtered = Vec::new();
        for (k, v) in all_items {
            let after_start = match &start {
                Bound::Included(s) => &k >= s,
                Bound::Excluded(s) => &k > s,
                Bound::Unbounded => true,
            };
            let before_end = match &end {
                Bound::Included(e) => &k <= e,
                Bound::Excluded(e) => &k < e,
                Bound::Unbounded => true,
            };
            if after_start && before_end {
                filtered.push((k, v));
            }
        }
        RangeIter {
            items: filtered,
            index: 0,
        }
    }
}

impl<K: Ord + Clone, V: Clone> Iterator for RangeIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let item = self.items[self.index].clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Create a range iterator over collected items.
pub fn range_query<K: Ord + Clone, V: Clone>(
    all_items: Vec<(K, V)>,
    start: Bound<K>,
    end: Bound<K>,
) -> RangeIter<K, V> {
    RangeIter::new(all_items, start, end)
}

#[cfg(test)]
mod tests {
    use crate::tree::BTree;

    #[test]
    fn test_range_full() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i * 10);
        }
        let range: Vec<_> = tree.range(..).collect();
        assert_eq!(range.len(), 10);
    }

    #[test]
    fn test_range_inclusive() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i);
        }
        let range: Vec<_> = tree.range(3..=7).collect();
        assert_eq!(range.len(), 5);
        assert_eq!(range[0].0, 3);
        assert_eq!(range[4].0, 7);
    }

    #[test]
    fn test_range_exclusive() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i);
        }
        let range: Vec<_> = tree.range(2..8).collect();
        assert_eq!(range.len(), 6);
    }

    #[test]
    fn test_range_open_end() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i);
        }
        let range: Vec<_> = tree.range(5..).collect();
        assert_eq!(range.len(), 5);
    }

    #[test]
    fn test_range_open_start() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i);
        }
        let range: Vec<_> = tree.range(..5).collect();
        assert_eq!(range.len(), 5);
    }

    #[test]
    fn test_range_empty() {
        let mut tree = BTree::new(3);
        for i in 0..5 {
            tree.insert(i, i);
        }
        let range: Vec<_> = tree.range(10..20).collect();
        assert!(range.is_empty());
    }

    #[test]
    fn test_range_single_element() {
        let mut tree = BTree::new(3);
        for i in 0..10 {
            tree.insert(i, i * 100);
        }
        let range: Vec<_> = tree.range(5..=5).collect();
        assert_eq!(range, vec![(5, 500)]);
    }
}
