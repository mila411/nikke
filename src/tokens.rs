#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    Boolean(bool),
    Null,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Comma,
    LeftParen,
    RightParen,
    Dot,
    Keyword(String),
}

pub fn is_keyword(literal: &str) -> bool {
    matches!(
        literal.to_uppercase().as_str(),
        "SELECT"
            | "INSERT"
            | "INTO"
            | "VALUES"
            | "FROM"
            | "JOIN"
            | "ON"
            | "WHERE"
            | "GROUP"
            | "BY"
            | "HAVING"
            | "ORDER"
            | "ASC"
            | "DESC"
            | "AND"
            | "OR"
            | "NOT"
    )
}

pub fn is_boolean(literal: &str) -> bool {
    literal.eq_ignore_ascii_case("TRUE") || literal.eq_ignore_ascii_case("FALSE")
}
