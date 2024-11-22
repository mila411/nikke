use crate::ast::{Expression, Insert, Query, Value};
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    KeywordSelect,
    KeywordInsert,
    KeywordInto,
    KeywordValues,
    KeywordFrom,
    KeywordWhere,
    KeywordAnd,
    KeywordOr,
    KeywordNull,
    KeywordTrue,
    KeywordFalse,
    KeywordDate,
    KeywordTime,
    Identifier(String),
    Comma,
    Asterisk,
    LeftParen,
    RightParen,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    SemiColon,
    StringLiteral(String),
    Float(f64),
    Integer(i64),
    HexInteger(i64),
    OctalInteger(i64),
    Comment(String),
    Whitespace(String),
}

// Lexer structure. Parses the input string into tokens.
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// Create a new Lexer.
    pub fn new(input_str: &'a str) -> Self {
        Lexer {
            input: input_str.chars().peekable(),
        }
    }

    /// The input string is broken down into a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                tokens.push(self.consume_whitespace());
            } else if c == '-' && self.peek_next() == Some('-') {
                tokens.push(self.consume_single_line_comment());
            } else if c == '/' && self.peek_next() == Some('*') {
                tokens.push(self.consume_multi_line_comment());
            } else if c.is_alphabetic() || c == '_' {
                tokens.push(self.consume_identifier_or_keyword());
            } else if c.is_digit(10) {
                let token = self.consume_number()?;
                tokens.push(token);
            } else {
                match c {
                    ',' => {
                        self.input.next();
                        tokens.push(Token::Comma);
                    }
                    '*' => {
                        self.input.next();
                        tokens.push(Token::Asterisk);
                    }
                    '(' => {
                        self.input.next();
                        tokens.push(Token::LeftParen);
                    }
                    ')' => {
                        self.input.next();
                        tokens.push(Token::RightParen);
                    }
                    '=' => {
                        self.input.next();
                        tokens.push(Token::Equal);
                    }
                    '!' => {
                        self.input.next();
                        if self.input.peek() == Some(&'=') {
                            self.input.next();
                            tokens.push(Token::NotEqual);
                        } else {
                            return Err("An invalid token '!' was detected.".to_string());
                        }
                    }
                    '>' => {
                        self.input.next();
                        tokens.push(Token::GreaterThan);
                    }
                    '<' => {
                        self.input.next();
                        tokens.push(Token::LessThan);
                    }
                    ';' => {
                        self.input.next();
                        tokens.push(Token::SemiColon);
                    }
                    '\'' => {
                        tokens.push(self.consume_string_literal());
                    }
                    _ => {
                        return Err(format!("Unknown character '{}' detected", c));
                    }
                }
            }
        }

        Ok(tokens)
    }

    /// will peek at the next character, but I will not advance the position.
    fn peek_next(&mut self) -> Option<char> {
        let mut iter = self.input.clone();
        iter.next()
    }

    /// Consumes whitespace characters and returns a whitespace token.
    fn consume_whitespace(&mut self) -> Token {
        let mut whitespace = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_whitespace() {
                whitespace.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        Token::Whitespace(whitespace)
    }

    /// Consumes a single-line comment and returns a Comment token.
    fn consume_single_line_comment(&mut self) -> Token {
        let mut comment = String::new();
        // Consumes '--'
        self.input.next();
        self.input.next();
        while let Some(&c) = self.input.peek() {
            if c == '\n' {
                break;
            }
            comment.push(c);
            self.input.next();
        }
        Token::Comment(comment)
    }

    /// Consumes a multi-line comment and returns a Comment token.
    fn consume_multi_line_comment(&mut self) -> Token {
        let mut comment = String::new();
        // Consume '/*'
        self.input.next();
        self.input.next();
        while let Some(&c) = self.input.peek() {
            if c == '*' && self.peek_next() == Some('/') {
                self.input.next();
                self.input.next();
                break;
            } else {
                comment.push(c);
                self.input.next();
            }
        }
        Token::Comment(comment)
    }

    /// It consumes an identifier or keyword and returns the corresponding token.
    fn consume_identifier_or_keyword(&mut self) -> Token {
        let mut identifier = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.input.next();
            } else {
                break;
            }
        }

        match identifier.to_uppercase().as_str() {
            "SELECT" => Token::KeywordSelect,
            "INSERT" => Token::KeywordInsert,
            "INTO" => Token::KeywordInto,
            "VALUES" => Token::KeywordValues,
            "FROM" => Token::KeywordFrom,
            "WHERE" => Token::KeywordWhere,
            "AND" => Token::KeywordAnd,
            "OR" => Token::KeywordOr,
            "NULL" => Token::KeywordNull,
            "TRUE" => Token::KeywordTrue,
            "FALSE" => Token::KeywordFalse,
            "DATE" => Token::KeywordDate,
            "TIME" => Token::KeywordTime,
            _ => Token::Identifier(identifier),
        }
    }

    /// Consumes a numeric literal and returns an Integer, Float, HexInteger, or OctalInteger token.
    fn consume_number(&mut self) -> Result<Token, String> {
        let mut number = String::new();

        // Check: 0x or 0o prefix
        if self.input.peek() == Some(&'0') {
            let mut iter = self.input.clone();
            iter.next(); // '0'

            if let Some(c) = iter.next() {
                if c == 'x' || c == 'X' {
                    // hexadecimal
                    self.input.next(); // '0'
                    self.input.next(); // 'x' または 'X'
                    return self.consume_hex_number();
                } else if c == 'o' || c == 'O' {
                    // Octal
                    self.input.next(); // '0'
                    self.input.next(); // 'o' または 'O'
                    return self.consume_oct_number();
                }
            }
        }

        // Normal decimal or floating-point number
        while let Some(&c) = self.input.peek() {
            if c.is_digit(10) {
                number.push(c);
                self.input.next();
            } else {
                break;
            }
        }

        // Checking floating-point numbers
        if self.input.peek() == Some(&'.') {
            number.push('.');
            self.input.next();
            while let Some(&c) = self.input.peek() {
                if c.is_digit(10) {
                    number.push(c);
                    self.input.next();
                } else {
                    break;
                }
            }
            // Optional support for exponential notation
            if self.input.peek() == Some(&'e') || self.input.peek() == Some(&'E') {
                number.push(self.input.next().unwrap());
                if let Some(&c) = self.input.peek() {
                    if c == '+' || c == '-' {
                        number.push(c);
                        self.input.next();
                    }
                }
                while let Some(&c) = self.input.peek() {
                    if c.is_digit(10) {
                        number.push(c);
                        self.input.next();
                    } else {
                        break;
                    }
                }
            }
            match number.parse::<f64>() {
                Ok(f) => Ok(Token::Float(f)),
                Err(_) => Err("Invalid floating-point literal".to_string()),
            }
        } else {
            // Integerリテラル
            match number.parse::<i64>() {
                Ok(i) => Ok(Token::Integer(i)),
                Err(_) => Err("Invalid integer literal".to_string()),
            }
        }
    }

    /// Consumes a hexadecimal literal and returns a HexInteger token.
    fn consume_hex_number(&mut self) -> Result<Token, String> {
        let mut hex_number = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_digit(16) {
                hex_number.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        match i64::from_str_radix(&hex_number, 16) {
            Ok(num) => Ok(Token::HexInteger(num)),
            Err(_) => Err(format!("Invalid hexadecimal literal: 0x{}", hex_number)),
        }
    }

    /// Consumes an octal literal and returns an OctalInteger token.
    fn consume_oct_number(&mut self) -> Result<Token, String> {
        let mut oct_number = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_digit(8) {
                oct_number.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        match i64::from_str_radix(&oct_number, 8) {
            Ok(num) => Ok(Token::OctalInteger(num)),
            Err(_) => Err(format!("Invalid octal literal: 0o{}", oct_number)),
        }
    }

    /// Consumes a string literal and returns a StringLiteral token.
    fn consume_string_literal(&mut self) -> Token {
        self.input.next();
        let mut string = String::new();
        while let Some(&c) = self.input.peek() {
            if c == '\'' {
                self.input.next();
                break;
            } else {
                string.push(c);
                self.input.next();
            }
        }
        Token::StringLiteral(string)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    /// Create a new parser. Tokenize the input string.
    pub fn new(query_str: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(query_str);
        let tokens = lexer.tokenize()?;
        println!("Token: {:?}", tokens); // For debugging
        Ok(Parser {
            tokens,
            position: 0,
        })
    }

    /// Obtain the next token from the current position of the parser.
    fn next_token(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    /// This refers to the token for the current position of the purser.
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Consumes whitespace and comments.
    fn consume_whitespace_and_comments(&mut self) {
        while let Some(token) = self.peek_token() {
            match token {
                Token::Whitespace(_) | Token::Comment(_) => {
                    self.position += 1;
                }
                _ => break,
            }
        }
    }

    /// The main function of the purser.
    pub fn parse(&mut self) -> Result<Query, String> {
        self.consume_whitespace_and_comments();

        match self.peek_token() {
            Some(Token::KeywordInsert) => self.parse_insert(),
            Some(Token::KeywordSelect) => self.parse_select(),
            _ => Err("Unknown query type".to_string()),
        }
    }

    /// Parses the INSERT statement.
    fn parse_insert(&mut self) -> Result<Query, String> {
        self.consume_whitespace_and_comments();

        // Consumption of INSERT keywords
        if !self.match_keyword("INSERT") {
            return Err("The INSERT keyword is required.".to_string());
        }

        self.consume_whitespace_and_comments();

        // Consumption of INTO keywords
        if !self.match_keyword("INTO") {
            return Err("INTO Keyword is required".to_string());
        }

        self.consume_whitespace_and_comments();

        // Parsing table names
        let table = if let Some(Token::Identifier(name)) = self.next_token() {
            name.clone()
        } else {
            return Err("Failed to parse table name".to_string());
        };

        self.consume_whitespace_and_comments();

        // Consumption of left parentheses
        if !self.match_token(&Token::LeftParen) {
            return Err("Left parentheses are required.".to_string());
        }

        self.consume_whitespace_and_comments();

        // Parsing column names
        let mut columns = Vec::new();
        loop {
            self.consume_whitespace_and_comments();
            if let Some(Token::Identifier(col)) = self.next_token() {
                columns.push(col.clone());
            } else {
                return Err("Failed to parse column name".to_string());
            }

            self.consume_whitespace_and_comments();

            if self.match_token(&Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        self.consume_whitespace_and_comments();

        // Consumption of right parentheses
        if !self.match_token(&Token::RightParen) {
            return Err("A right parenthesis is required.".to_string());
        }

        self.consume_whitespace_and_comments();

        // Consumption of VALUES keywords
        if !self.match_keyword("VALUES") {
            return Err("VALUES Keyword is required".to_string());
        }

        self.consume_whitespace_and_comments();

        // Consumption of left parentheses
        if !self.match_token(&Token::LeftParen) {
            return Err("Left parentheses are required.".to_string());
        }

        self.consume_whitespace_and_comments();

        // Value Parsing
        let mut values = Vec::new();
        loop {
            self.consume_whitespace_and_comments();
            match self.next_token() {
                Some(Token::Integer(i)) => values.push(Value::Integer(*i)),
                Some(Token::HexInteger(i)) => values.push(Value::Integer(*i)),
                Some(Token::OctalInteger(i)) => values.push(Value::Integer(*i)),
                Some(Token::Float(f)) => values.push(Value::Float(*f)),
                Some(Token::StringLiteral(s)) => values.push(Value::Text(s.clone())),
                Some(Token::KeywordNull) => values.push(Value::Null),
                Some(Token::KeywordTrue) => values.push(Value::Boolean(true)),
                Some(Token::KeywordFalse) => values.push(Value::Boolean(false)),
                Some(Token::KeywordDate) => {
                    if let Some(Token::StringLiteral(s)) = self.next_token() {
                        values.push(Value::Date(s.clone()));
                    } else {
                        return Err("Failed to parse DATE literal".to_string());
                    }
                }
                Some(Token::KeywordTime) => {
                    if let Some(Token::StringLiteral(s)) = self.next_token() {
                        values.push(Value::Time(s.clone()));
                    } else {
                        return Err("Failed to parse TIME literal".to_string());
                    }
                }
                _ => return Err("Failed to parse value".to_string()),
            }

            self.consume_whitespace_and_comments();

            if self.match_token(&Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        self.consume_whitespace_and_comments();

        // Consumption of right parentheses
        if !self.match_token(&Token::RightParen) {
            return Err("A right parenthesis is required.".to_string());
        }

        self.consume_whitespace_and_comments();

        // Consumption of optional semicolons
        self.match_token(&Token::SemiColon);

        Ok(Query::Insert(Insert {
            table,
            columns,
            values,
        }))
    }

    /// The SELECT statement is parsed (implementation omitted).
    fn parse_select(&mut self) -> Result<Query, String> {
        Err("The SELECT parser is not yet implemented.".to_string())
    }

    /// Match the keywords.
    fn match_keyword(&mut self, keyword: &str) -> bool {
        if let Some(token) = self.peek_token() {
            let token_str = match token {
                Token::KeywordSelect => "SELECT",
                Token::KeywordInsert => "INSERT",
                Token::KeywordInto => "INTO",
                Token::KeywordValues => "VALUES",
                Token::KeywordFrom => "FROM",
                Token::KeywordWhere => "WHERE",
                Token::KeywordAnd => "AND",
                Token::KeywordOr => "OR",
                Token::KeywordNull => "NULL",
                Token::KeywordTrue => "TRUE",
                Token::KeywordFalse => "FALSE",
                Token::KeywordDate => "DATE",
                Token::KeywordTime => "TIME",
                _ => "",
            };
            if keyword.to_uppercase() == token_str {
                self.position += 1;
                return true;
            }
        }
        false
    }

    /// Match the tokens.
    fn match_token(&mut self, expected: &Token) -> bool {
        if let Some(token) = self.peek_token() {
            if token == expected {
                self.position += 1;
                return true;
            }
        }
        false
    }

    /// Parses the WHERE clause expression.
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.consume_whitespace_and_comments();

        // Left-hand side parse
        let left = if let Some(Token::Identifier(left)) = self.next_token() {
            left.clone()
        } else {
            return Err("The left side of the WHERE clause is invalid.".to_string());
        };

        self.consume_whitespace_and_comments();

        // Operator Parsing
        let operator = if self.match_token(&Token::Equal) {
            "="
        } else if self.match_token(&Token::NotEqual) {
            "!="
        } else if self.match_token(&Token::GreaterThan) {
            ">"
        } else if self.match_token(&Token::LessThan) {
            "<"
        } else {
            return Err("The operator in the WHERE clause is invalid.".to_string());
        };

        self.consume_whitespace_and_comments();

        // Right-hand side parsing
        let right = match self.next_token() {
            Some(Token::Integer(i)) => Value::Integer(*i),
            Some(Token::HexInteger(i)) => Value::Integer(*i),
            Some(Token::OctalInteger(i)) => Value::Integer(*i),
            Some(Token::Float(f)) => Value::Float(*f),
            Some(Token::StringLiteral(s)) => Value::Text(s.clone()),
            Some(Token::KeywordNull) => Value::Null,
            Some(Token::KeywordTrue) => Value::Boolean(true),
            Some(Token::KeywordFalse) => Value::Boolean(false),
            Some(Token::KeywordDate) => {
                if let Some(Token::StringLiteral(s)) = self.next_token() {
                    Value::Date(s.clone())
                } else {
                    return Err("Failed to parse DATE literal".to_string());
                }
            }
            Some(Token::KeywordTime) => {
                if let Some(Token::StringLiteral(s)) = self.next_token() {
                    Value::Time(s.clone())
                } else {
                    return Err("Failed to parse TIME literal".to_string());
                }
            }
            _ => return Err("The right side of the WHERE clause is invalid.".to_string()),
        };

        match operator {
            "=" => Ok(Expression::Equals(left, right)),
            "!=" => Ok(Expression::NotEquals(left, right)),
            ">" => Ok(Expression::GreaterThan(left, right)),
            "<" => Ok(Expression::LessThan(left, right)),
            _ => Err("The operator in the WHERE clause is invalid.".to_string()),
        }
    }
}
