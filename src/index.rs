use crate::buffer_pool::BufferPool;
use crate::storage::{NodeType, Page};
use std::sync::{Arc, RwLock};

// Re-export Key and Value types to make them publicly accessible
pub use crate::storage::{Key, Value};

/// Represents a B+ Tree structure.
pub struct BPlusTree {
    root: Arc<RwLock<Option<Arc<Page>>>>,
    buffer_pool: BufferPool,
}

impl BPlusTree {
    /// Initializes a new B+ Tree with the given storage file.
    pub fn new(file_path: &str) -> std::io::Result<Self> {
        let storage = crate::storage::StorageEngine::new(file_path)?;
        let buffer_pool = BufferPool::new(100, storage);
        Ok(BPlusTree {
            root: Arc::new(RwLock::new(None)),
            buffer_pool,
        })
    }

    /// Inserts a key-value pair into the B+ Tree.
    pub fn insert(&self, key: Key, value: Value) -> std::io::Result<()> {
        // Acquire read lock on root
        let root_option = {
            let root_read = self.root.read().unwrap();
            root_read.clone()
        };

        if root_option.is_none() {
            // Tree is empty, create a new leaf node
            let new_leaf = self.buffer_pool.allocate_page(NodeType::Leaf)?;
            {
                let mut leaf_data = new_leaf.data.write().unwrap();
                leaf_data.keys.push(key);
                leaf_data.values.push(value);
            }
            // Write the modified page back to the buffer pool
            self.buffer_pool.write_page(&new_leaf)?;

            // Set the new leaf as the root
            let mut root_write = self.root.write().unwrap();
            *root_write = Some(new_leaf);
            return Ok(());
        }

        // Find the appropriate leaf node for the key
        let leaf = self.find_leaf_page(key)?;
        let need_split = {
            let mut leaf_data = leaf.data.write().unwrap();
            let pos = leaf_data.keys.binary_search(&key).unwrap_or_else(|e| e);
            leaf_data.keys.insert(pos, key);
            leaf_data.values.insert(pos, value);

            // Write the modified leaf back to the buffer pool
            self.buffer_pool.write_page(&leaf)?;

            leaf_data.keys.len() > ORDER - 1
        };

        if need_split {
            self.split_leaf_page(leaf)?;
        }

        Ok(())
    }

    /// Searches for a value by its key in the B+ Tree.
    pub fn search(&self, key: Key) -> std::io::Result<Option<Value>> {
        let root_option = {
            let root_read = self.root.read().unwrap();
            root_read.clone()
        };

        if root_option.is_none() {
            return Ok(None);
        }

        let leaf = self.find_leaf_page(key)?;
        let leaf_data = leaf.data.read().unwrap();

        match leaf_data.keys.binary_search(&key) {
            Ok(idx) => Ok(Some(leaf_data.values[idx])),
            Err(_) => Ok(None),
        }
    }

    /// Finds the appropriate leaf page for a given key.
    fn find_leaf_page(&self, key: Key) -> std::io::Result<Arc<Page>> {
        let mut current_option = {
            let root_read = self.root.read().unwrap();
            root_read.clone()
        };

        while let Some(current) = current_option {
            let node_type = {
                let current_data = current.data.read().unwrap();
                current_data.node_type.clone()
            }; // Borrow ended here

            match node_type {
                NodeType::Leaf => {
                    return Ok(current); // Safe to move `current` here
                }
                NodeType::Internal => {
                    let child_id = {
                        let current_data = current.data.read().unwrap();
                        let idx = match current_data.keys.binary_search(&key) {
                            Ok(idx) => idx + 1,
                            Err(idx) => idx,
                        };

                        if idx >= current_data.children.len() {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Invalid child index: {}", idx),
                            ));
                        }

                        current_data.children[idx]
                    }; // Borrow ended here

                    current_option = Some(self.buffer_pool.get_page(child_id)?);
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "The tree is empty",
        ))
    }

    /// Splits a leaf page that has exceeded its capacity.
    fn split_leaf_page(&self, leaf: Arc<Page>) -> std::io::Result<()> {
        let new_leaf = self.buffer_pool.allocate_page(NodeType::Leaf)?;
        let up_key;

        {
            let mut leaf_data = leaf.data.write().unwrap();
            let mut new_leaf_data = new_leaf.data.write().unwrap();
            let mid = leaf_data.keys.len() / 2;

            new_leaf_data.keys = leaf_data.keys.split_off(mid);
            new_leaf_data.values = leaf_data.values.split_off(mid);
            new_leaf_data.next = leaf_data.next.take();
            new_leaf_data.parent_id = leaf_data.parent_id;

            up_key = new_leaf_data.keys[0];

            leaf_data.next = Some(new_leaf_data.id);

            // Write both leaf and new_leaf back to the buffer pool
            self.buffer_pool.write_page(&leaf)?;
            self.buffer_pool.write_page(&new_leaf)?;
        }

        self.insert_into_parent(leaf, up_key, new_leaf)
    }

    /// Inserts a key and a new child into the parent node after splitting.
    fn insert_into_parent(
        &self,
        left: Arc<Page>,
        key: Key,
        right: Arc<Page>,
    ) -> std::io::Result<()> {
        let parent_id = {
            let left_data = left.data.read().unwrap();
            left_data.parent_id
        };

        if let Some(parent_id) = parent_id {
            let parent = self.buffer_pool.get_page(parent_id)?;
            let need_split = {
                let mut parent_data = parent.data.write().unwrap();
                let pos = parent_data.keys.binary_search(&key).unwrap_or_else(|e| e);
                parent_data.keys.insert(pos, key);
                parent_data
                    .children
                    .insert(pos + 1, right.data.read().unwrap().id);
                right.data.write().unwrap().parent_id = Some(parent_data.id);

                // Write the modified parent back to the buffer pool
                self.buffer_pool.write_page(&parent)?;

                parent_data.keys.len() > ORDER - 1
            };

            if need_split {
                self.split_internal_page(parent)?;
            }
            Ok(())
        } else {
            // No parent exists, create a new root
            let new_root = self.buffer_pool.allocate_page(NodeType::Internal)?;
            {
                let mut new_root_data = new_root.data.write().unwrap();
                new_root_data.keys.push(key);
                new_root_data.children.push(left.data.read().unwrap().id);
                new_root_data.children.push(right.data.read().unwrap().id);
                left.data.write().unwrap().parent_id = Some(new_root_data.id);
                right.data.write().unwrap().parent_id = Some(new_root_data.id);
            }
            // Write the new root back to the buffer pool
            self.buffer_pool.write_page(&new_root)?;
            let mut root_write = self.root.write().unwrap();
            *root_write = Some(new_root);
            Ok(())
        }
    }

    /// Splits an internal node that has exceeded its capacity.
    fn split_internal_page(&self, node: Arc<Page>) -> std::io::Result<()> {
        let new_internal = self.buffer_pool.allocate_page(NodeType::Internal)?;
        let up_key;

        {
            let mut node_data = node.data.write().unwrap();
            let mut new_internal_data = new_internal.data.write().unwrap();
            let mid = node_data.keys.len() / 2;

            up_key = node_data.keys[mid];

            new_internal_data.keys = node_data.keys.split_off(mid + 1);
            new_internal_data.children = node_data.children.split_off(mid + 1);

            for &child_id in &new_internal_data.children {
                let child = self.buffer_pool.get_page(child_id)?;
                child.data.write().unwrap().parent_id = Some(new_internal_data.id);
            }

            new_internal_data.parent_id = node_data.parent_id;

            node_data.keys.truncate(mid);
            node_data.children.truncate(mid + 1);

            // Write both node and new_internal back to the buffer pool
            self.buffer_pool.write_page(&node)?;
            self.buffer_pool.write_page(&new_internal)?;
        }

        self.insert_into_parent(node, up_key, new_internal)
    }
}

// B+ Tree order (degree)
pub const ORDER: usize = 4;

#[cfg(test)]
mod tests {
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
        let tree = BPlusTree::new(test_db).expect("Failed to initialize storage engine");

        println!("Inserting key-value pairs...");
        // Insert key-value pairs
        for i in 0..100 {
            println!("Inserting key: {}, value: {}", i, i * 10);
            tree.insert(i, (i * 10) as u64)
                .expect("Failed to insert key-value pair");
            if i % 10 == 0 {
                println!("Inserted {} key-value pairs so far.", i + 1);
            }
        }

        println!("All key-value pairs inserted successfully.");

        println!("Searching for inserted keys...");
        // Search for the inserted keys
        for i in 0..100 {
            let result = tree.search(i).expect("Failed to search for key");
            assert_eq!(result, Some((i * 10) as u64));
            if i % 10 == 0 {
                println!("Searched {} keys so far.", i + 1);
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
        let tree = Arc::new(BPlusTree::new(test_db).expect("Failed to initialize storage engine"));

        // Spawn multiple threads to perform insert operations
        let mut insert_handles = vec![];
        for i in 0..4 {
            let tree_clone = Arc::clone(&tree);

            let handle = thread::spawn(move || {
                for j in 0..25 {
                    let key = i * 25 + j;
                    println!("Thread {} inserting key: {}, value: {}", i, key, key * 10);
                    tree_clone
                        .insert(key, (key * 10) as u64)
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
                    assert_eq!(result, Some((key * 10) as u64));
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
