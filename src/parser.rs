// src/parser.rs

use crate::ast::{Expression, Insert, Query, Select, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Date(String),
    Time(String),
    Timestamp(String),
    Interval(String),
    Star,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    Comma,
    SemiColon,
    Whitespace(String),
    Illegal(String), // For invalid tokens
                     // Add other tokens as needed
}

// Lexer struct responsible for tokenizing the input string
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,      // Current position in input (points to current char)
    read_position: usize, // Current reading position in input (after current char)
    ch: Option<char>,     // Current char under examination
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer instance and initializes the first character.
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char(); // Initialize the first character
        lexer
    }

    /// Reads the next character and advances positions.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None; // End of input
        } else {
            self.ch = Some(self.input[self.read_position..].chars().next().unwrap());
        }
        self.position = self.read_position;
        if let Some(c) = self.ch {
            self.read_position += c.len_utf8();
        }
    }

    /// Peeks at the next character without consuming it.
    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            self.input[self.read_position..].chars().next()
        }
    }

    /// Skips over any whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.ch {
            if !c.is_whitespace() {
                break;
            }
            self.read_char();
        }
    }

    /// Reads a string literal enclosed in single quotes.
    fn read_string_literal(&mut self) -> String {
        self.read_char(); // Consume the opening quote
        let mut literal = String::new();
        while let Some(c) = self.ch {
            if c == '\'' {
                break;
            }
            literal.push(c);
            self.read_char();
        }
        self.read_char(); // Consume the closing quote
        literal
    }

    /// Reads a numeric literal (integer, float, hexadecimal, or octal).
    fn read_number(&mut self) -> Token {
        if let Some('0') = self.ch {
            // Possible hexadecimal or octal
            if let Some(next_ch) = self.peek_char() {
                if next_ch == 'x' || next_ch == 'X' {
                    // Hexadecimal
                    self.read_char(); // Consume '0'
                    self.read_char(); // Consume 'x' or 'X'
                    return self.read_hex_number();
                } else if next_ch == 'o' || next_ch == 'O' {
                    // Octal
                    self.read_char(); // Consume '0'
                    self.read_char(); // Consume 'o' or 'O'
                    return self.read_octal_number();
                }
            }
        }

        // Existing number parsing (integer or float)
        let mut number = String::new();
        let mut has_decimal_point = false;
        while let Some(c) = self.ch {
            if c == '.' {
                if has_decimal_point {
                    break; // Second decimal point encountered
                }
                has_decimal_point = true;
                number.push(c);
            } else if c == 'e' || c == 'E' {
                number.push(c);
                self.read_char();
                if let Some(next_c) = self.ch {
                    if next_c == '+' || next_c == '-' {
                        number.push(next_c);
                        self.read_char();
                    }
                }
                continue;
            } else if c.is_ascii_digit() {
                number.push(c);
            } else {
                break;
            }
            self.read_char();
        }

        if has_decimal_point || number.contains('e') || number.contains('E') {
            if let Ok(f) = number.parse::<f64>() {
                Token::Float(f)
            } else {
                Token::Illegal(number)
            }
        } else {
            if let Ok(i) = number.parse::<i64>() {
                Token::Integer(i)
            } else {
                Token::Illegal(number)
            }
        }
    }

    /// Reads a hexadecimal number.
    fn read_hex_number(&mut self) -> Token {
        let mut number = String::new();
        while let Some(c) = self.ch {
            if !c.is_digit(16) {
                break;
            }
            number.push(c);
            self.read_char();
        }
        if let Ok(i) = i64::from_str_radix(&number, 16) {
            Token::Integer(i)
        } else {
            Token::Illegal(format!("0x{}", number))
        }
    }

    /// Reads an octal number.
    fn read_octal_number(&mut self) -> Token {
        let mut number = String::new();
        while let Some(c) = self.ch {
            if c < '0' || c > '7' {
                break;
            }
            number.push(c);
            self.read_char();
        }
        if let Ok(i) = i64::from_str_radix(&number, 8) {
            Token::Integer(i)
        } else {
            Token::Illegal(format!("0o{}", number))
        }
    }

    /// Reads an identifier or keyword.
    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(c) = self.ch {
            if !Self::is_identifier_part(c) {
                break;
            }
            ident.push(c);
            self.read_char();
        }
        match ident.to_uppercase().as_str() {
            "INSERT" | "INTO" | "VALUES" | "SELECT" | "FROM" | "WHERE" | "AND" | "OR" | "NOT"
            | "DATE" | "TIME" | "TIMESTAMP" | "INTERVAL" | "NULL" | "TRUE" | "FALSE" => {
                Token::Keyword(ident.to_uppercase())
            }
            _ => Token::Identifier(ident),
        }
    }

    /// Checks if a character can start an identifier.
    fn is_identifier_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /// Checks if a character can be part of an identifier.
    fn is_identifier_part(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Returns the next token from the input.
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let token = match self.ch {
            Some('(') => {
                self.read_char();
                Token::LeftParen
            }
            Some(')') => {
                self.read_char();
                Token::RightParen
            }
            Some(',') => {
                self.read_char();
                Token::Comma
            }
            Some(';') => {
                self.read_char();
                Token::SemiColon
            }
            Some('*') => {
                self.read_char();
                Token::Star
            }
            Some('=') => {
                self.read_char();
                Token::Equal
            }
            Some('!') => {
                self.read_char();
                if let Some('=') = self.ch {
                    self.read_char();
                    Token::NotEqual
                } else {
                    Token::Not
                }
            }
            Some('<') => {
                self.read_char();
                if let Some('=') = self.ch {
                    self.read_char();
                    Token::LessThanOrEqual
                } else {
                    Token::LessThan
                }
            }
            Some('>') => {
                self.read_char();
                if let Some('=') = self.ch {
                    self.read_char();
                    Token::GreaterThanOrEqual
                } else {
                    Token::GreaterThan
                }
            }
            Some('\'') => Token::StringLiteral(self.read_string_literal()),
            Some(c) if c.is_ascii_digit() => self.read_number(),
            Some(c) if Self::is_identifier_start(c) => self.read_identifier_or_keyword(),
            Some(_) => {
                // Handle unknown characters
                let illegal_char = self.ch.unwrap();
                self.read_char();
                Token::Illegal(illegal_char.to_string())
            }
            None => {
                // End of input
                return None;
            }
        };
        Some(token)
    }
}

// Parser struct responsible for parsing tokens into an abstract syntax tree
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance.
    pub fn new(input: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let first_token = lexer.next_token();
        Ok(Parser {
            lexer,
            current_token: first_token,
        })
    }

    /// Advances to the next token.
    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    /// Matches and consumes the current token if it matches the expected token.
    fn match_token(&mut self, token: &Token) -> bool {
        if let Some(ref current) = self.current_token {
            if current == token {
                self.next_token();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Matches and consumes a keyword.
    fn match_keyword(&mut self, keyword: &str) -> bool {
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

    /// Parses the entire query.
    pub fn parse(&mut self) -> Result<Query, String> {
        match self.current_token {
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("INSERT") => {
                self.parse_insert()
            }
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("SELECT") => {
                self.parse_select()
            }
            _ => Err("Unsupported query type.".to_string()),
        }
    }

    /// Parses an INSERT statement.
    fn parse_insert(&mut self) -> Result<Query, String> {
        // Consume 'INSERT'
        if !self.match_keyword("INSERT") {
            return Err("Expected 'INSERT' keyword.".to_string());
        }

        // Consume 'INTO'
        if !self.match_keyword("INTO") {
            return Err("Expected 'INTO' keyword.".to_string());
        }

        // Parse table name
        let table = if let Some(Token::Identifier(ref name)) = self.current_token {
            let table_name = name.clone();
            self.next_token();
            table_name
        } else {
            return Err("Expected table name.".to_string());
        };

        // Consume '('
        if !self.match_token(&Token::LeftParen) {
            return Err("Expected '('.".to_string());
        }

        // Parse column names
        let mut columns = Vec::new();
        loop {
            if let Some(Token::Identifier(ref col)) = self.current_token {
                columns.push(col.clone());
                self.next_token();
            } else {
                return Err("Expected column name.".to_string());
            }

            if self.match_token(&Token::Comma) {
                continue;
            } else if self.match_token(&Token::RightParen) {
                break;
            } else {
                return Err("Expected ',' or ')'.".to_string());
            }
        }

        // Consume 'VALUES'
        if !self.match_keyword("VALUES") {
            return Err("Expected 'VALUES' keyword.".to_string());
        }

        // Consume '('
        if !self.match_token(&Token::LeftParen) {
            return Err("Expected '('.".to_string());
        }

        // Parse values
        let mut values = Vec::new();
        loop {
            self.consume_whitespace_and_comments();

            let value = self.parse_value()?;

            values.push(value);

            self.consume_whitespace_and_comments();

            if self.match_token(&Token::Comma) {
                continue;
            } else if self.match_token(&Token::RightParen) {
                break;
            } else {
                return Err("Expected ',' or ')'.".to_string());
            }
        }

        // Consume optional ';'
        self.match_token(&Token::SemiColon);

        Ok(Query::Insert(Insert {
            table,
            columns,
            values,
        }))
    }

    /// Parses a SELECT statement.
    fn parse_select(&mut self) -> Result<Query, String> {
        // Consume 'SELECT'
        if !self.match_keyword("SELECT") {
            return Err("Expected 'SELECT' keyword.".to_string());
        }

        // Parse column list
        let mut columns = Vec::new();
        loop {
            match &self.current_token {
                Some(Token::Star) => {
                    columns.push("*".to_string());
                    self.next_token();
                }
                Some(Token::Identifier(ident)) => {
                    columns.push(ident.clone());
                    self.next_token();
                }
                _ => return Err("Expected '*' or column name.".to_string()),
            }

            if self.match_token(&Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        // Consume 'FROM'
        if !self.match_keyword("FROM") {
            return Err("Expected 'FROM' keyword.".to_string());
        }

        // Parse table name
        let table = if let Some(Token::Identifier(ref name)) = self.current_token {
            let table_name = name.clone();
            self.next_token();
            table_name
        } else {
            return Err("Expected table name.".to_string());
        };

        // Optional 'WHERE' clause
        let where_clause = if self.match_keyword("WHERE") {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Consume optional ';'
        self.match_token(&Token::SemiColon);

        Ok(Query::Select(Select {
            columns,
            table,
            where_clause,
        }))
    }

    /// Parses an expression for the WHERE clause.
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let left = match self.current_token.clone() {
            Some(Token::Identifier(ident)) => {
                self.next_token();
                ident
            }
            _ => return Err("Expected identifier.".to_string()),
        };

        // Parse operator
        let operator = match self.current_token.clone() {
            Some(Token::Equal) => {
                self.next_token();
                "=".to_string()
            }
            Some(Token::NotEqual) => {
                self.next_token();
                "!=".to_string()
            }
            Some(Token::LessThan) => {
                self.next_token();
                "<".to_string()
            }
            Some(Token::LessThanOrEqual) => {
                self.next_token();
                "<=".to_string()
            }
            Some(Token::GreaterThan) => {
                self.next_token();
                ">".to_string()
            }
            Some(Token::GreaterThanOrEqual) => {
                self.next_token();
                ">=".to_string()
            }
            _ => return Err("Expected comparison operator.".to_string()),
        };

        // Parse right-hand side value
        let right = self.parse_value()?;

        Ok(Expression::Binary {
            left,
            operator,
            right,
        })
    }

    /// Parses a value used in expressions.
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
            Some(Token::StringLiteral(s)) => {
                self.next_token();
                Ok(Value::Text(s))
            }
            Some(Token::Null) => {
                self.next_token();
                Ok(Value::Null)
            }
            Some(Token::Boolean(b)) => {
                self.next_token();
                Ok(Value::Boolean(b))
            }
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("DATE") => {
                self.next_token();
                if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                    self.next_token();
                    Ok(Value::Date(s))
                } else {
                    Err("Failed to parse 'DATE' literal.".to_string())
                }
            }
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("TIME") => {
                self.next_token();
                if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                    self.next_token();
                    Ok(Value::Time(s))
                } else {
                    Err("Failed to parse 'TIME' literal.".to_string())
                }
            }
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("TIMESTAMP") => {
                self.next_token();
                if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                    self.next_token();
                    Ok(Value::Timestamp(s))
                } else {
                    Err("Failed to parse 'TIMESTAMP' literal.".to_string())
                }
            }
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("INTERVAL") => {
                self.next_token();
                if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                    self.next_token();
                    Ok(Value::Interval(s))
                } else {
                    Err("Failed to parse 'INTERVAL' literal.".to_string())
                }
            }
            Some(Token::Illegal(s)) => Err(format!("Illegal token encountered: {}", s)),
            _ => Err("Failed to parse value.".to_string()),
        }
    }

    /// Consumes any whitespace and comments.
    fn consume_whitespace_and_comments(&mut self) {
        while let Some(Token::Whitespace(_)) = self.current_token {
            self.next_token();
        }
        // Add comment handling if necessary
    }
}
