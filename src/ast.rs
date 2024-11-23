// src/ast.rs

#[derive(Debug)]
pub enum Query {
    Insert(Insert),
    Select(Select),
    // Add other query types like Update, Delete, etc.
}

#[derive(Debug)]
pub struct Insert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

#[derive(Debug)]
pub struct Select {
    pub columns: Vec<String>,
    pub table: String,
    pub where_clause: Option<Expression>,
}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Null,
    Date(String),
    Time(String),
    Timestamp(String),
    Interval(String),
    // Add other value types as needed
}

#[derive(Debug)]
pub enum Expression {
    Binary {
        left: String,
        operator: String,
        right: Value,
    },
    // Add more expression types as needed
}
