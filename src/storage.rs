use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::RwLock;

/// Type alias for keys in the B+ Tree.
pub type Key = i32;

/// Type alias for values in the B+ Tree.
pub type Value = u64;

/// Enum representing the type of a B+ Tree node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Internal,
    Leaf,
}

/// Fixed page size (4KB).
pub const PAGE_SIZE: usize = 4096;

/// Data stored within a page.
#[derive(Debug, Serialize, Deserialize)]
pub struct PageData {
    pub id: u32,
    pub node_type: NodeType,
    pub keys: Vec<Key>,
    pub children: Vec<u32>, // Child page IDs
    pub values: Vec<Value>,
    pub next: Option<u32>,      // Next leaf page ID
    pub parent_id: Option<u32>, // Parent page ID
}

impl PageData {
    /// Creates a new PageData instance.
    pub fn new(id: u32, node_type: NodeType) -> Self {
        PageData {
            id,
            node_type,
            keys: Vec::new(),
            children: Vec::new(),
            values: Vec::new(),
            next: None,
            parent_id: None,
        }
    }
}

/// Represents a page with its data protected by a read-write lock.
pub struct Page {
    pub data: RwLock<PageData>,
}

impl Page {
    /// Creates a new Page instance.
    pub fn new(id: u32, node_type: NodeType) -> Self {
        Page {
            data: RwLock::new(PageData::new(id, node_type)),
        }
    }
}

/// StorageEngine manages reading and writing pages to disk.
pub struct StorageEngine {
    file: File,
}

impl StorageEngine {
    /// Creates a new StorageEngine with the given file path.
    pub fn new(file_path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)?;
        Ok(StorageEngine { file })
    }

    /// Reads a page from disk by its ID.
    pub fn read_page(&mut self, page_id: u32) -> std::io::Result<PageData> {
        let mut buffer = vec![0u8; PAGE_SIZE];
        self.file
            .seek(SeekFrom::Start(page_id as u64 * PAGE_SIZE as u64))?;
        self.file.read_exact(&mut buffer)?;

        // Deserialize the page data
        let page_data: PageData = bincode::deserialize(&buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(page_data)
    }

    /// Writes a page to disk.
    pub fn write_page(&mut self, page_data: &PageData) -> std::io::Result<()> {
        // Serialize the page data
        let encoded: Vec<u8> = bincode::serialize(page_data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if encoded.len() > PAGE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Page size exceeded",
            ));
        }

        // Pad the buffer to PAGE_SIZE
        let mut buffer = encoded;
        buffer.resize(PAGE_SIZE, 0u8);

        self.file
            .seek(SeekFrom::Start(page_data.id as u64 * PAGE_SIZE as u64))?;
        self.file.write_all(&buffer)?;
        Ok(())
    }

    /// Allocates a new page with the specified node type.
    pub fn allocate_page(&mut self, node_type: NodeType) -> std::io::Result<PageData> {
        let page_id = (self.file.metadata()?.len() / PAGE_SIZE as u64) as u32;
        let page_data = PageData::new(page_id, node_type);
        self.write_page(&page_data)?;
        Ok(page_data)
    }
}
