#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Null,
    Date(String),
    Time(String),
}

#[derive(Debug)]
pub enum Expression {
    Equals(String, Value),
    NotEquals(String, Value),
    GreaterThan(String, Value),
    LessThan(String, Value),
}

#[derive(Debug)]
pub enum Query {
    Select(Select),
    Insert(Insert),
    // Add future query types
}

#[derive(Debug)]
pub struct Select {
    pub columns: Vec<String>,
    pub table: String,
    pub where_clause: Option<Expression>,
    // Add other fields (e.g. ORDER BY, GROUP BY)
}

#[derive(Debug)]
pub struct Insert {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}
