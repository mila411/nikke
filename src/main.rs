// src/main.rs

mod ast;
mod buffer_pool;
mod index;
mod parser;
mod storage;

use buffer_pool::BufferPool;
use index::BPlusTree;
use parser::Parser;
use std::io::{self, Write};
use std::sync::Arc;
use storage::{Key, Value};

fn main() {
    println!("Please select a test.:");
    println!("1. B+ Tree Input Test");
    println!("2. SQL Parser Input Test");
    print!("Please enter the number of your choice (1 or 2): ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin()
        .read_line(&mut choice)
        .expect("Input reading failed.");

    match choice.trim() {
        "1" => run_b_plus_tree_input_test(),
        "2" => run_sql_parser_input_test(),
        _ => println!("Invalid selection. Please enter 1 or 2."),
    }
}

fn run_b_plus_tree_input_test() {
    println!("B+ Tree input test in progress...");

    // Database file for testing
    let test_db = "test.db";
    let storage_engine =
        storage::StorageEngine::new(test_db).expect("Failed to create StorageEngine");
    let buffer_pool = Arc::new(BufferPool::new(100, storage_engine));
    let tree = BPlusTree::new(buffer_pool.clone(), 4).expect("Failed to create B+ tree.");
    let tree = Arc::new(tree);

    loop {
        println!("Please select an operation.:");
        println!("1. Insert key and value");
        println!("2. Search for a key");
        println!("3. Exit");
        print!("Please enter the number of your choice (1, 2 or 3): ");
        io::stdout().flush().unwrap();

        let mut operation = String::new();
        io::stdin()
            .read_line(&mut operation)
            .expect("Input reading failed.");

        match operation.trim() {
            "1" => {
                print!("Please enter the key to insert.: ");
                io::stdout().flush().unwrap();

                let mut key_str = String::new();
                io::stdin()
                    .read_line(&mut key_str)
                    .expect("Input reading failed.");

                let key_str = key_str.trim();
                let key: Key = match key_str.parse() {
                    Ok(k) => k,
                    Err(_) => {
                        println!("Invalid key. Please enter an integer.");
                        continue;
                    }
                };

                print!("Please enter the value to be inserted.: ");
                io::stdout().flush().unwrap();

                let mut value_str = String::new();
                io::stdin()
                    .read_line(&mut value_str)
                    .expect("Input reading failed.");

                let value_str = value_str.trim();
                let value: Value = match value_str.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        println!("Invalid value. Please enter a whole number.");
                        continue;
                    }
                };

                tree.insert(key, value).expect("Failed to insert key");
                println!("The key {} and value {} have been inserted.", key, value);
            }
            "2" => {
                print!("Please enter the key you are searching for.: ");
                io::stdout().flush().unwrap();

                let mut key_str = String::new();
                io::stdin()
                    .read_line(&mut key_str)
                    .expect("Input reading failed.");

                let key_str = key_str.trim();
                let key: Key = match key_str.parse() {
                    Ok(k) => k,
                    Err(_) => {
                        println!("Invalid key. Please enter an integer.");
                        continue;
                    }
                };

                match tree.search(key) {
                    Ok(Some(value)) => println!("The value of key {} is {} .", key, value),
                    Ok(None) => println!("The key {} was not found.", key),
                    Err(e) => println!("An error occurred while searching.: {}", e),
                }
            }
            "3" => break,
            _ => println!("Invalid selection. Please enter 1, 2 or 3."),
        }
    }

    // Deleting the test database file
    let _ = std::fs::remove_file(test_db);
    println!("The B+ tree input test has been completed.");
}

fn run_sql_parser_input_test() {
    println!("Running SQL parser input test...");

    loop {
        print!("Please enter SQL (to exit, enter 'exit').: ");
        io::stdout().flush().unwrap(); // Flash to display prompts

        let mut query_str = String::new();
        io::stdin()
            .read_line(&mut query_str)
            .expect("Input reading failed.");

        let query_str = query_str.trim();
        if query_str.eq_ignore_ascii_case("exit") {
            break;
        }

        let mut parser = match Parser::new(query_str) {
            Ok(p) => p,
            Err(e) => {
                println!("Lexer error: {}", e);
                continue;
            }
        };

        match parser.parse() {
            Ok(query) => println!("Analysis results: {:?}", query),
            Err(e) => println!("Parsing error: {}", e),
        }
    }

    println!("The SQL parser input test has been completed.");
}
