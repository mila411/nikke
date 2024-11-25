use crate::ast::{
    BinaryOperator, Expression, Insert, Join, Ordering, Query, Select, SortOrder, Table, Value,
};
use crate::lexer::Lexer;
use crate::tokens::Token;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Create a new parser.
    pub fn new(input: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let first_token = lexer.next_token();
        Ok(Parser {
            lexer,
            current_token: first_token,
        })
    }

    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        if let Some(Token::Keyword(ref kw)) = self.current_token {
            if kw.eq_ignore_ascii_case(keyword) {
                self.next_token();
                Ok(())
            } else {
                Err(format!(
                    "Expected keyword '{}', but found '{}'",
                    keyword, kw
                ))
            }
        } else {
            Err(format!("Expected keyword '{}'", keyword))
        }
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if let Some(Token::Keyword(ref kw)) = self.current_token {
            if kw.eq_ignore_ascii_case(keyword) {
                self.next_token();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn consume_keywords(&mut self, keywords: &[&str]) -> bool {
        let original_token = self.current_token.clone();
        for &keyword in keywords {
            if !self.consume_keyword(keyword) {
                self.current_token = original_token.clone();
                return false;
            }
        }
        true
    }

    fn peek_keyword(&self, keyword: &str) -> bool {
        if let Some(Token::Keyword(ref kw)) = self.current_token {
            kw.eq_ignore_ascii_case(keyword)
        } else {
            false
        }
    }

    fn expect_token(&mut self, expected: &Token) -> Result<(), String> {
        if let Some(ref current) = self.current_token {
            if current == expected {
                self.next_token();
                Ok(())
            } else {
                Err(format!(
                    "Expected token '{:?}', but found '{:?}'",
                    expected, current
                ))
            }
        } else {
            Err(format!("Expected token '{:?}', but reached EOF", expected))
        }
    }

    fn consume_token(&mut self, expected: &Token) -> bool {
        if let Some(ref current) = self.current_token {
            if current == expected {
                self.next_token();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// The entire query is parsed.
    pub fn parse(&mut self) -> Result<Query, String> {
        if self.peek_keyword("SELECT") {
            self.parse_select()
        } else if self.peek_keyword("INSERT") {
            self.parse_insert()
        } else {
            Err("This is an unsupported query type.".to_string())
        }
    }

    /// Parses the INSERT statement.
    fn parse_insert(&mut self) -> Result<Query, String> {
        self.expect_keyword("INSERT")?;
        self.expect_keyword("INTO")?;
        let table = self.parse_table()?;

        self.expect_token(&Token::LeftParen)?;
        let mut columns = Vec::new();
        loop {
            if let Some(Token::Identifier(ref col)) = self.current_token {
                columns.push(col.clone());
                self.next_token();
            } else {
                return Err("I was expecting a column name.".to_string());
            }

            if !self.consume_token(&Token::Comma) {
                break;
            }
        }
        self.expect_token(&Token::RightParen)?;

        if self.consume_keyword("VALUES") {
            self.expect_token(&Token::LeftParen)?;
            let mut values = Vec::new();
            loop {
                let value = self.parse_value()?;
                values.push(value);

                if !self.consume_token(&Token::Comma) {
                    break;
                }
            }
            self.expect_token(&Token::RightParen)?;

            Ok(Query::Insert(Insert {
                table,
                columns,
                values: Some(values),
                select: None,
            }))
        } else if self.peek_keyword("SELECT") {
            let select = self.parse_select_inner()?;
            Ok(Query::Insert(Insert {
                table,
                columns,
                values: None,
                select: Some(Box::new(select)),
            }))
        } else {
            Err("'VALUES' or 'SELECT' is required after the column.".to_string())
        }
    }

    /// Parse the SELECT statement and wrap it in `Query::Select`.
    fn parse_select(&mut self) -> Result<Query, String> {
        let select = self.parse_select_inner()?;
        Ok(Query::Select(select))
    }

    /// A function that parses SELECT statements internally
    fn parse_select_inner(&mut self) -> Result<Select, String> {
        self.expect_keyword("SELECT")?;
        let mut columns = Vec::new();
        loop {
            columns.push(self.parse_expression()?);
            if !self.consume_token(&Token::Comma) {
                break;
            }
        }

        self.expect_keyword("FROM")?;
        let (table, joins) = self.parse_table_with_joins()?;

        let where_clause = if self.consume_keyword("WHERE") {
            Some(self.parse_logical_expression()?)
        } else {
            None
        };

        let group_by = if self.consume_keywords(&["GROUP", "BY"]) {
            Some(self.parse_group_by_clause()?)
        } else {
            None
        };

        let having = if self.consume_keyword("HAVING") {
            Some(self.parse_logical_expression()?)
        } else {
            None
        };

        let order_by = if self.consume_keywords(&["ORDER", "BY"]) {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        Ok(Select {
            columns,
            table,
            joins,
            where_clause,
            group_by,
            having,
            order_by,
        })
    }

    fn parse_table_with_joins(&mut self) -> Result<(Table, Vec<Join>), String> {
        let table = self.parse_table()?;
        let mut joins = Vec::new();
        while self.peek_keyword("JOIN") {
            let join = self.parse_join_clause()?;
            joins.push(join);
        }
        Ok((table, joins))
    }

    fn parse_table(&mut self) -> Result<Table, String> {
        if let Some(Token::Identifier(ref name)) = self.current_token {
            let table = Table { name: name.clone() };
            self.next_token();
            Ok(table)
        } else {
            Err("I was expecting a table name".to_string())
        }
    }

    fn parse_join_clause(&mut self) -> Result<Join, String> {
        self.expect_keyword("JOIN")?;
        let table = self.parse_table()?;
        let condition = if self.consume_keyword("ON") {
            Some(self.parse_logical_expression()?)
        } else {
            None
        };
        Ok(Join { table, condition })
    }

    fn parse_logical_expression(&mut self) -> Result<Expression, String> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and_expression()?;
        while self.consume_keyword("OR") {
            let right = self.parse_and_expression()?;
            expr = Expression::Or(Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_and_expression(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_not_expression()?;
        while self.consume_keyword("AND") {
            let right = self.parse_not_expression()?;
            expr = Expression::And(Box::new(expr), Box::new(right));
        }
        Ok(expr)
    }

    fn parse_not_expression(&mut self) -> Result<Expression, String> {
        if self.consume_keyword("NOT") {
            let expr = self.parse_primary_expression()?;
            Ok(Expression::Not(Box::new(expr)))
        } else {
            self.parse_primary_expression()
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression, String> {
        if self.consume_token(&Token::LeftParen) {
            let expr = self.parse_logical_expression()?;
            self.expect_token(&Token::RightParen)?;
            Ok(expr)
        } else {
            self.parse_comparison_expression()
        }
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_term()?;
        if let Some(op) = self.current_token.clone() {
            let operator = match op {
                Token::Equal => Some(BinaryOperator::Equal),
                Token::NotEqual => Some(BinaryOperator::NotEqual),
                Token::LessThan => Some(BinaryOperator::LessThan),
                Token::LessThanOrEqual => Some(BinaryOperator::LessThanOrEqual),
                Token::GreaterThan => Some(BinaryOperator::GreaterThan),
                Token::GreaterThanOrEqual => Some(BinaryOperator::GreaterThanOrEqual),
                _ => None,
            };

            if let Some(op) = operator {
                self.next_token();
                let right = self.parse_term()?;
                Ok(Expression::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                })
            } else {
                Ok(left)
            }
        } else {
            Ok(left)
        }
    }

    fn parse_group_by_clause(&mut self) -> Result<Vec<Expression>, String> {
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.parse_expression()?);
            if !self.consume_token(&Token::Comma) {
                break;
            }
        }
        Ok(expressions)
    }

    fn parse_order_by_clause(&mut self) -> Result<Vec<Ordering>, String> {
        let mut orderings = Vec::new();
        loop {
            let expr = self.parse_expression()?;
            let direction = if self.consume_keyword("ASC") {
                SortOrder::Ascending
            } else if self.consume_keyword("DESC") {
                SortOrder::Descending
            } else {
                SortOrder::Ascending
            };
            orderings.push(Ordering {
                expression: expr,
                direction,
            });
            if !self.consume_token(&Token::Comma) {
                break;
            }
        }
        Ok(orderings)
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_logical_expression()
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        match self.current_token.clone() {
            Some(Token::Integer(i)) => {
                self.next_token();
                Ok(Value::Integer(i))
            }
            Some(Token::Float(f)) => {
                self.next_token();
                Ok(Value::Float(f))
            }
            Some(Token::StringLiteral(ref s)) => {
                self.next_token();
                Ok(Value::Text(s.clone()))
            }
            Some(Token::Null) => {
                self.next_token();
                Ok(Value::Null)
            }
            Some(Token::Boolean(b)) => {
                self.next_token();
                Ok(Value::Boolean(b))
            }
            _ => Err("This is an unexpected token.".to_string()),
        }
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        match self.current_token.clone() {
            Some(Token::Identifier(ref name)) => {
                let identifier = name.clone();
                self.next_token();
                if self.consume_token(&Token::Dot) {
                    if let Some(Token::Identifier(ref field)) = self.current_token {
                        let field_name = format!("{}.{}", identifier, field);
                        self.next_token();
                        Ok(Expression::Identifier(field_name))
                    } else {
                        Err("I was expecting a field name.".to_string())
                    }
                } else if self.consume_token(&Token::LeftParen) {
                    let mut args = Vec::new();
                    if !self.consume_token(&Token::RightParen) {
                        loop {
                            let expr = self.parse_expression()?;
                            args.push(expr);
                            if self.consume_token(&Token::Comma) {
                                continue;
                            } else {
                                self.expect_token(&Token::RightParen)?;
                                break;
                            }
                        }
                    }
                    Ok(Expression::Function(identifier, args))
                } else {
                    Ok(Expression::Identifier(identifier))
                }
            }
            Some(Token::Integer(i)) => {
                self.next_token();
                Ok(Expression::Integer(i))
            }
            Some(Token::Float(f)) => {
                self.next_token();
                Ok(Expression::Float(f))
            }
            Some(Token::StringLiteral(ref s)) => {
                self.next_token();
                Ok(Expression::Text(s.clone()))
            }
            Some(Token::Null) => {
                self.next_token();
                Ok(Expression::Identifier("NULL".to_string()))
            }
            Some(Token::Boolean(b)) => {
                self.next_token();
                Ok(Expression::Boolean(b))
            }
            Some(Token::Asterisk) => {
                self.next_token();
                Ok(Expression::Asterisk)
            }
            _ => Err("This is an unexpected token.".to_string()),
        }
    }
}
