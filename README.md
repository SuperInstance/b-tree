# b-tree

A pure-Rust B-tree implementation with insertion, deletion, search, and range queries.

## Features

- Configurable order (branching factor)
- Sorted key-value storage
- Range query support with inclusive/exclusive bounds
- In-order traversal and depth invariant verification
- Zero external dependencies

## Usage

```rust
use b_tree::BTree;

let mut tree = BTree::new(3); // order 3 (max 5 keys per node)
tree.insert(10, "hello");
tree.insert(20, "world");
assert_eq!(tree.search(&10), Some(&"hello"));

// Range query
let range: Vec<_> = tree.range(..20).collect();
```

## Modules

- `node` — B-tree node structure and operations
- `tree` — Top-level B-tree container
- `insert` — Insertion and split logic
- `delete` — Deletion, rebalancing, and merge logic
- `range` — Range query iteration
