#[derive(Debug)]
pub enum Expression {
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Identifier(String),
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Function(String, Vec<Expression>),
}

#[derive(Debug)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug)]
pub struct Insert {
    pub table: Table,
    pub columns: Vec<String>,
    pub values: Option<Vec<Value>>,
    pub select: Option<Box<Select>>,
}

#[derive(Debug)]
pub struct Join {
    pub table: Table,
    pub condition: Option<Expression>,
}

#[derive(Debug)]
pub struct Ordering {
    pub expression: Expression,
    pub direction: SortOrder,
}

#[derive(Debug)]
pub enum Query {
    Select(Select),
    Insert(Insert),
}

#[derive(Debug)]
pub struct Select {
    pub columns: Vec<Expression>,
    pub table: Table,
    pub joins: Vec<Join>,
    pub where_clause: Option<Expression>,
    pub group_by: Option<Vec<Expression>>,
    pub having: Option<Expression>,
    pub order_by: Option<Vec<Ordering>>,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
}

#[derive(Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Null,
}
