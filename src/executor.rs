// A file for preparing for the future. I hope you can get this far.

use crate::ast::{ASTNode, Statement};
use crate::storage::Storage;
use crate::transaction::TransactionManager;

// Query execution engine
pub struct Executor {
    storage: Storage,
    tx_manager: TransactionManager,
}

impl Executor {
    pub fn new(storage: Storage, tx_manager: TransactionManager) -> Self {
        Executor {
            storage,
            tx_manager,
        }
    }

    // Execute AST node
    pub fn execute(&mut self, ast: ASTNode) -> Result<(), String> {
        match ast {
            ASTNode::Statement(stmt) => self.execute_statement(stmt),
        }
    }

    // Executing a statement
    fn execute_statement(&mut self, stmt: Statement) -> Result<(), String> {
        match stmt {
            Statement::CreateTable { name, columns } => {
                println!("Creates a table '{}'.", name);
                Ok(())
            }
            Statement::Insert { table, values } => {
                println!("Insert data into table '{}'", table);
                Ok(())
            }
            Statement::Select { table, columns } => {
                println!("Select data from table '{}'", table);
                Ok(())
            } // other statements
        }
    }
}
