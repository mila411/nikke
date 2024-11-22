/*
TODO: I thought I had implemented it with the utmost care so that it wouldn't cause a deadlock, but there are some parts that seem to be causing a deadlock when I run the unit tests.
*/

use crate::storage::{NodeType, Page, StorageEngine};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// BufferPool manages cached pages with LRU eviction policy.
pub struct BufferPool {
    capacity: usize,
    // Combined pool and LRU queue under a single Mutex to prevent deadlocks
    pool_and_lru: Mutex<PoolAndLRU>,
    storage: Mutex<StorageEngine>,
}

struct PoolAndLRU {
    pool: HashMap<u32, Arc<Page>>,
    lru_queue: VecDeque<u32>,
}

impl BufferPool {
    /// Creates a new BufferPool with specified capacity and storage engine.
    pub fn new(capacity: usize, storage: StorageEngine) -> Self {
        BufferPool {
            capacity,
            pool_and_lru: Mutex::new(PoolAndLRU {
                pool: HashMap::new(),
                lru_queue: VecDeque::new(),
            }),
            storage: Mutex::new(storage),
        }
    }

    /// Retrieves a page by its ID. If not cached, loads from storage.
    pub fn get_page(&self, page_id: u32) -> std::io::Result<Arc<Page>> {
        println!("BufferPool::get_page - Requested page_id: {}", page_id);

        // Attempt to get the page from the pool
        {
            let mut pool_lru = self.pool_and_lru.lock().unwrap();
            if let Some(page) = pool_lru.pool.get(&page_id).cloned() {
                println!(
                    "BufferPool::get_page - Page {} found in pool. Updating LRU.",
                    page_id
                );
                // Move the accessed page to the front of the LRU queue
                if let Some(pos) = pool_lru.lru_queue.iter().position(|&id| id == page_id) {
                    pool_lru.lru_queue.remove(pos);
                }
                pool_lru.lru_queue.push_front(page_id);
                return Ok(page);
            } else {
                println!("BufferPool::get_page - Page {} not found in pool.", page_id);
            }
        }

        // If not in pool, load from storage
        println!(
            "BufferPool::get_page - Loading page {} from storage.",
            page_id
        );
        let page_data = {
            // Unified lock acquisition order: lock storage after locking pool_and_lru
            let _pool_lru = self.pool_and_lru.lock().unwrap();
            let mut storage_lock = self.storage.lock().unwrap();
            storage_lock.read_page(page_id)?
        };

        let page_id_new = page_data.id; // Extract the id before moving
        let page = Arc::new(Page {
            data: std::sync::RwLock::new(page_data),
        });

        println!(
            "BufferPool::get_page - Inserting new page {} into pool.",
            page_id_new
        );
        // Insert the new page into the pool
        {
            let mut pool_lru = self.pool_and_lru.lock().unwrap();

            // Evict least recently used page if capacity is exceeded
            if pool_lru.pool.len() >= self.capacity {
                if let Some(old_id) = pool_lru.lru_queue.pop_back() {
                    println!(
                        "BufferPool::get_page - Evicting least recently used page {}.",
                        old_id
                    );
                    pool_lru.pool.remove(&old_id);
                }
            }

            pool_lru.pool.insert(page_id_new, Arc::clone(&page));
            pool_lru.lru_queue.push_front(page_id_new);
        }

        Ok(page)
    }

    /// Writes a page back to storage.
    pub fn write_page(&self, page: &Page) -> std::io::Result<()> {
        let page_id = page.data.read().unwrap().id;
        println!(
            "BufferPool::write_page - Writing page {} to storage.",
            page_id
        );
        let page_data = page.data.read().unwrap();
        let mut storage = self.storage.lock().unwrap();
        storage.write_page(&page_data)
    }

    /// Allocates a new page and inserts it into the pool.
    pub fn allocate_page(&self, node_type: NodeType) -> std::io::Result<Arc<Page>> {
        println!(
            "BufferPool::allocate_page - Allocating new page of type {:?}",
            node_type
        );

        // Unify the order of obtaining locks: lock pool_and_lru first
        let mut pool_lru = self.pool_and_lru.lock().unwrap();

        // Allocate the page in storage
        let page_data = {
            let mut storage_lock = self.storage.lock().unwrap();
            storage_lock.allocate_page(node_type)?
        };

        let page_id_new = page_data.id; // Extract the id before moving
        let page = Arc::new(Page {
            data: std::sync::RwLock::new(page_data),
        });

        println!(
            "BufferPool::allocate_page - Inserting new page {} into pool.",
            page_id_new
        );
        // Insert the new page into the pool

        // Evict least recently used page if capacity is exceeded
        if pool_lru.pool.len() >= self.capacity {
            if let Some(old_id) = pool_lru.lru_queue.pop_back() {
                println!(
                    "BufferPool::allocate_page - Evicting least recently used page {}.",
                    old_id
                );
                pool_lru.pool.remove(&old_id);
            }
        }

        pool_lru.pool.insert(page_id_new, Arc::clone(&page));
        pool_lru.lru_queue.push_front(page_id_new);

        Ok(page)
    }
}
