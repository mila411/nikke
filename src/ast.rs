#[derive(Debug)]
pub enum Query {
    Select(Select),
    Insert(Insert),
    // 他のクエリタイプも追加可能
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
pub struct Insert {
    pub table: Table,
    pub columns: Vec<String>,
    pub values: Option<Vec<Value>>,
    pub select: Option<Box<Select>>, // INSERT INTO ... SELECT ... をサポート
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
}

#[derive(Debug)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Identifier(String),
    Function(String, Vec<Expression>), // COUNT関数などをサポート
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    // 他の式タイプも追加可能
}

#[derive(Debug)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    // 他の演算子も追加可能
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
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Null,
    Boolean(bool),
    Date(String),
    Time(String),
    Timestamp(String),
    Interval(String),
    // 他の値タイプも追加可能
}
