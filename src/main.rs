mod buffer_pool;
mod index;
pub mod storage;

use index::{BPlusTree, Key, Value};
use std::io::{self, Write};

fn main() {
    let file_path = "rusqlite.db";
    let tree = match BPlusTree::new(file_path) {
        Ok(t) => t,
        Err(e) => {
            println!("Failed to initialize the storage engine.: {}", e);
            return;
        }
    };

    println!("A simple demo using the B+ tree index");
    println!("Commands: insert <key> <value> | search <key> | exit");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("The input could not be read.");
            continue;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_lowercase().as_str() {
            "insert" => {
                if parts.len() != 3 {
                    println!("Usage: insert <key: i32> <value: u64>");
                    continue;
                }
                let key: Key = match parts[1].parse() {
                    Ok(k) => k,
                    Err(_) => {
                        println!("Please specify the key as i32.");
                        continue;
                    }
                };
                let value: Value = match parts[2].parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Please specify the value in u64 format.");
                        continue;
                    }
                };
                if let Err(e) = tree.insert(key, value) {
                    println!("Failed to insert: {}", e);
                } else {
                    println!("Inserted: key = {}, value = {}", key, value);
                }
            }
            "search" => {
                if parts.len() != 2 {
                    println!("Usage: search <key: i32>");
                    continue;
                }
                let key: Key = match parts[1].parse() {
                    Ok(k) => k,
                    Err(_) => {
                        println!("Please specify the key as i32.");
                        continue;
                    }
                };
                match tree.search(key) {
                    Ok(Some(value)) => println!("Search results: key = {}, value = {}", key, value),
                    Ok(None) => println!("The key {} does not exist.", key),
                    Err(e) => println!("Search failed.: {}", e),
                }
            }
            "exit" => {
                println!("It will end.");
                break;
            }
            _ => {
                println!("Unknown command. Usage: insert | search | exit");
            }
        }
    }
}
