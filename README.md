# nikke: Experimental RDBMS Project for Rust Learning

This project, named nikke, is an experimental and learning repository aimed at deepening my understanding of Rust through low-level programming. Inspired by SQLite, nikke focuses on building a lightweight RDBMS to explore the intricacies of Rust and systems programming.

## 🛠️ Features

### 📦 Data Storage Engine
- **Page Management using File I/O**: Efficiently handles data storage by managing data in fixed-size pages, enabling quick read and write operations.
- **Buffer Pool Implementation**: Introduces a buffer pool to cache frequently accessed pages, reducing disk I/O and improving performance.

### 🌳 B+ Tree Indexing
- **Efficient Data Searches**: Implements B+ trees to provide fast and reliable indexing mechanisms, ensuring quick data retrieval even as the dataset grows.

### 📝 SQL Parser and AST
- **Basic SQL Statement Parsing**: Parses fundamental SQL commands, converting them into Abstract Syntax Trees (AST) for further processing and execution.

### 🔍 Query Engine
- **Execution Plan Generation and Execution**: Transforms ASTs into executable plans, allowing for optimized query execution and data manipulation.

### 🔄 Transaction Management
- **Logging and Recovery Mechanisms**: Ensures data integrity and consistency through robust transaction logging and recovery processes, safeguarding against data loss and corruption.

### 🖥️ Command-Line Interface (CLI)
- **User Query Input Processing**: Provides a user-friendly CLI for interacting with the database, enabling users to input and execute SQL queries seamlessly.

## 🚧 Current Progress

nikke is steadily progressing towards its goal of becoming a fully-featured RDBMS, but the following components have already been implemented, and there is still a long way to go:

1. **Data Storage Engine**
   - [x] **Page Management using File I/O**
   - [x] **Buffer Pool Implementation**

2. **B+ Tree Indexing**
   - [x] **Efficient Data Searches**

3. **SQL Parser and AST**
   - [ ] **Basic SQL Statement Parsing**

4. **Query Engine**
   - [ ] **Execution Plan Generation and Execution**

5. **Transaction Management**
   - [ ] **Logging and Recovery Mechanisms**

6. **Command-Line Interface (CLI)**
   - [ ] **User Query Input Processing**

## 🎯 Future Goals

To transform nikke into a robust RDBMS akin to SQLite, the following milestones are planned:

- **Advanced SQL Support**: Extend the SQL parser to handle more complex queries, including JOINs, subqueries, and transactions.
- **Enhanced Indexing**: Implement additional indexing strategies such as hash indexes and full-text search.
- **Concurrency Control**: Improve transaction management with advanced concurrency control mechanisms like Multi-Version Concurrency Control (MVCC).
- **Performance Optimization**: Continuously optimize existing components for better performance and lower resource consumption.
- **Comprehensive Testing**: Develop a suite of unit and integration tests to ensure reliability and stability.
- **Documentation and Tutorials**: Provide detailed documentation and tutorials to help users understand and contribute to nikke.
