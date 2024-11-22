// A file for preparing for the future. I hope you can get this far.

// AST Node Definition
pub enum ASTNode {
    Statement(Statement),
}

// Enumeration of SQL statements
pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDefinition>,
    },
    Insert {
        table: String,
        values: Vec<String>,
    },
    Select {
        table: String,
        columns: Vec<String>,
    },
}

// Column definition
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
}

// Data type enumeration
pub enum DataType {
    Integer,
    Varchar,
}
