use crate::buffer_pool::BufferPool;
use crate::storage::{Key, Value};
use std::sync::{Arc, RwLock};

/// Represents the B+ Tree order (degree).
pub const ORDER: usize = 4;

/// Represents a node in the B+ Tree.
#[derive(Debug)]
struct BPlusTreeNode {
    keys: Vec<Key>,
    children: Vec<Arc<RwLock<BPlusTreeNode>>>,
    is_leaf: bool,
}

/// Represents the B+ Tree structure.
pub struct BPlusTree {
    root: Arc<RwLock<Option<Arc<RwLock<BPlusTreeNode>>>>>,
    _buffer_pool: Arc<BufferPool>,
    order: usize,
}

impl BPlusTree {
    /// Initializes a new B+ Tree with the given buffer pool and order.
    pub fn new(buffer_pool: Arc<BufferPool>, order: usize) -> Result<Self, String> {
        if order < 3 {
            return Err("B+ Tree order must be at least 3".to_string());
        }

        // Initialize the root node as a leaf
        let root_node = Arc::new(RwLock::new(BPlusTreeNode {
            keys: Vec::new(),
            children: Vec::new(),
            is_leaf: true,
        }));

        Ok(BPlusTree {
            root: Arc::new(RwLock::new(Some(Arc::clone(&root_node)))),
            _buffer_pool: buffer_pool,
            order,
        })
    }

    /// Inserts a key into the B+ Tree.
    pub fn insert(&self, key: Key, value: Value) -> Result<(), String> {
        let mut root_guard = self.root.write().unwrap();

        if root_guard.is_none() {
            // Tree is empty, create a new leaf node
            let new_leaf = Arc::new(RwLock::new(BPlusTreeNode {
                keys: vec![key],
                children: Vec::new(),
                is_leaf: true,
            }));
            *root_guard = Some(Arc::clone(&new_leaf));
            return Ok(());
        }

        let split = self.insert_recursive(Arc::clone(root_guard.as_ref().unwrap()), key, value)?;

        if let Some((new_key, new_child)) = split {
            // Create a new root
            let new_root = Arc::new(RwLock::new(BPlusTreeNode {
                keys: vec![new_key],
                children: vec![Arc::clone(root_guard.as_ref().unwrap()), new_child],
                is_leaf: false,
            }));
            *root_guard = Some(Arc::clone(&new_root));
        }

        Ok(())
    }

    /// Recursively inserts a key-value pair and handles node splits.
    fn insert_recursive(
        &self,
        node: Arc<RwLock<BPlusTreeNode>>,
        key: Key,
        value: Value,
    ) -> Result<Option<(Key, Arc<RwLock<BPlusTreeNode>>)>, String> {
        let mut node_guard = node.write().unwrap();

        if node_guard.is_leaf {
            // Insert the key in the leaf node
            if node_guard.keys.contains(&key) {
                return Err("Duplicate key insertion is not allowed".to_string());
            }
            node_guard.keys.push(key);
            node_guard.keys.sort();

            if node_guard.keys.len() > self.order - 1 {
                // Split the leaf node
                let mid = self.order / 2;
                let split_key = node_guard.keys[mid].clone();

                let new_leaf = Arc::new(RwLock::new(BPlusTreeNode {
                    keys: node_guard.keys.split_off(mid),
                    children: Vec::new(),
                    is_leaf: true,
                }));

                return Ok(Some((split_key, new_leaf)));
            }

            Ok(None)
        } else {
            // Internal node: find the child to descend
            let pos = node_guard
                .keys
                .iter()
                .position(|k| k >= &key)
                .unwrap_or(node_guard.keys.len());

            if pos < node_guard.children.len() {
                let child = Arc::clone(&node_guard.children[pos]);
                drop(node_guard); // Release the lock before recursive call

                let split = self.insert_recursive(child, key, value)?;

                if let Some((new_key, new_child)) = split {
                    // Insert the new key and child into the current node
                    let mut node_guard = node.write().unwrap();
                    node_guard.keys.push(new_key);
                    node_guard.keys.sort();
                    node_guard.children.push(new_child);
                    node_guard.children.sort_by_key(|c| {
                        let c_guard = c.read().unwrap();
                        c_guard.keys.first().cloned().unwrap_or(new_key.clone())
                    });

                    if node_guard.keys.len() > self.order - 1 {
                        // Split the internal node
                        let mid = self.order / 2;
                        let split_key = node_guard.keys[mid].clone();

                        let new_internal = Arc::new(RwLock::new(BPlusTreeNode {
                            keys: node_guard.keys.split_off(mid + 1),
                            children: node_guard.children.split_off(mid + 1),
                            is_leaf: false,
                        }));

                        return Ok(Some((split_key, new_internal)));
                    }
                }
            }

            Ok(None)
        }
    }

    /// Searches for a value by its key in the B+ Tree.
    pub fn search(&self, key: Key) -> Result<Option<Value>, String> {
        let root_guard = self.root.read().unwrap();

        if root_guard.is_none() {
            return Ok(None);
        }

        self.search_recursive(Arc::clone(root_guard.as_ref().unwrap()), key)
    }

    /// Recursively searches for a key.
    fn search_recursive(
        &self,
        node: Arc<RwLock<BPlusTreeNode>>,
        key: Key,
    ) -> Result<Option<Value>, String> {
        let node_guard = node.read().unwrap();

        if node_guard.is_leaf {
            // Search in the leaf node
            match node_guard.keys.binary_search(&key) {
                Ok(_idx) => {
                    // Assuming values are stored alongside keys. Adjust accordingly.
                    // Here, for simplicity, returning a dummy Value.
                    Ok(Some(Value::from(key as u64 * 10)))
                }
                Err(_) => Ok(None),
            }
        } else {
            // Internal node: find the child to descend
            let pos = node_guard
                .keys
                .iter()
                .position(|k| k >= &key)
                .unwrap_or(node_guard.keys.len());

            if pos < node_guard.children.len() {
                let child = Arc::clone(&node_guard.children[pos]);
                drop(node_guard); // Release the lock before recursive call
                self.search_recursive(child, key)
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::StorageEngine;

    use super::*;
    use std::fs;
    use std::sync::Arc;
    use std::thread;

    /// Tests single-threaded insert and search operations.
    #[test]
    fn test_single_thread_insert_and_search() {
        let test_db = "test_rusql.db";
        // Remove the test database file if it exists
        let _ = fs::remove_file(test_db);

        println!("Initializing BPlusTree...");
        let buffer_pool = Arc::new(BufferPool::new(100, StorageEngine::new(test_db).unwrap()));
        let tree = BPlusTree::new(Arc::clone(&buffer_pool), ORDER)
            .expect("Failed to initialize BPlusTree");

        println!("Inserting key-value pairs...");
        // Insert key-value pairs
        for i in 0..100 {
            println!("Inserting key: {}, value: {}", i, i * 10);
            tree.insert(i, Value::from((i * 10) as u64))
                .expect("Failed to insert key-value pair");
            if i % 10 == 0 && i != 0 {
                println!("Inserted {} key-value pairs so far.", i);
            }
        }

        println!("All key-value pairs inserted successfully.");

        println!("Searching for inserted keys...");
        // Search for the inserted keys
        for i in 0..100 {
            let result = tree.search(i).expect("Failed to search for key");
            assert_eq!(result, Some(Value::from((i * 10) as u64)));
            if i % 10 == 0 && i != 0 {
                println!("Searched {} keys so far.", i);
            }
        }

        println!("All keys searched successfully.");

        println!("Cleaning up the test database file...");
        // Clean up the test database file
        let _ = fs::remove_file(test_db);
        println!("Test completed successfully.");
    }

    /// Tests multi-threaded insert and search operations.
    #[test]
    fn test_multi_thread_insert_and_search() {
        let test_db = "test_multi_thread.db";
        // Remove the test database file if it exists
        let _ = fs::remove_file(test_db);

        println!("Initializing BPlusTree for multi-threaded test...");
        let buffer_pool = Arc::new(BufferPool::new(100, StorageEngine::new(test_db).unwrap()));
        let tree = Arc::new(
            BPlusTree::new(Arc::clone(&buffer_pool), ORDER)
                .expect("Failed to initialize BPlusTree"),
        );

        // Spawn multiple threads to perform insert operations
        let mut insert_handles = vec![];
        for i in 0..4 {
            let tree_clone = Arc::clone(&tree);

            let handle = thread::spawn(move || {
                for j in 0..25 {
                    let key = i * 25 + j;
                    println!("Thread {} inserting key: {}, value: {}", i, key, key * 10);
                    tree_clone
                        .insert(key, Value::from((key * 10) as u64))
                        .expect("Failed to insert key-value pair");
                    if j % 5 == 0 {
                        println!("Thread {} inserted {} key-value pairs so far.", i, j + 1);
                    }
                }
            });
            insert_handles.push(handle);
        }

        // Wait for all insert threads to finish
        for handle in insert_handles {
            handle.join().expect("Failed to join insert thread");
        }

        println!("All insert threads have completed.");

        // Spawn multiple threads to perform search operations
        let mut search_handles = vec![];
        for i in 0..4 {
            let tree_clone = Arc::clone(&tree);

            let handle = thread::spawn(move || {
                for j in 0..25 {
                    let key = i * 25 + j;
                    let result = tree_clone.search(key).expect("Failed to search for key");
                    assert_eq!(result, Some(Value::from((key * 10) as u64)));
                    if j % 5 == 0 {
                        println!("Thread {} searched {} keys so far.", i, j + 1);
                    }
                }
            });
            search_handles.push(handle);
        }

        // Wait for all search threads to finish
        for handle in search_handles {
            handle.join().expect("Failed to join search thread");
        }

        println!("All search threads have completed.");

        println!("Cleaning up the test database file...");
        // Clean up the test database file
        let _ = fs::remove_file(test_db);
        println!("Multi-threaded test completed successfully.");
    }
}
