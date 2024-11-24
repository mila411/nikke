use crate::tokens::{is_boolean, is_keyword, Token};
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Chars<'a>,
    current_char: Option<char>,
    peek_char: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut l = Lexer {
            chars: input.chars(),
            current_char: None,
            peek_char: None,
        };
        l.read_char();
        l.read_char_peek();
        l
    }

    fn read_char(&mut self) {
        self.current_char = self.chars.next();
    }

    fn read_char_peek(&mut self) {
        self.peek_char = self.chars.clone().next();
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let token = match self.current_char {
            Some(c) if c.is_alphabetic() => self.read_identifier(),
            Some(c) if c.is_digit(10) => self.read_number(),
            Some('\'') => self.read_string_literal(),
            Some('=') => {
                self.read_char();
                Some(Token::Equal)
            }
            Some('!') => {
                if self.peek_char == Some('=') {
                    self.read_char();
                    self.read_char();
                    Some(Token::NotEqual)
                } else {
                    None
                }
            }
            Some('<') => {
                if self.peek_char == Some('=') {
                    self.read_char();
                    self.read_char();
                    Some(Token::LessThanOrEqual)
                } else {
                    self.read_char();
                    Some(Token::LessThan)
                }
            }
            Some('>') => {
                if self.peek_char == Some('=') {
                    self.read_char();
                    self.read_char();
                    Some(Token::GreaterThanOrEqual)
                } else {
                    self.read_char();
                    Some(Token::GreaterThan)
                }
            }
            Some(',') => {
                self.read_char();
                Some(Token::Comma)
            }
            Some('(') => {
                self.read_char();
                Some(Token::LeftParen)
            }
            Some(')') => {
                self.read_char();
                Some(Token::RightParen)
            }
            Some('.') => {
                self.read_char();
                Some(Token::Dot)
            }
            Some(_c) => {
                self.read_char();
                None
            }
            None => None,
        };

        token
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Option<Token> {
        let mut identifier = String::new();
        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.read_char();
            } else {
                break;
            }
        }

        if is_keyword(&identifier) {
            Some(Token::Keyword(identifier.to_uppercase()))
        } else if is_boolean(&identifier) {
            Some(Token::Boolean(identifier.eq_ignore_ascii_case("TRUE")))
        } else {
            Some(Token::Identifier(identifier))
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        let mut number = String::new();
        while let Some(c) = self.current_char {
            if c.is_digit(10) {
                number.push(c);
                self.read_char();
            } else {
                break;
            }
        }

        if self.current_char == Some('.') {
            number.push('.');
            self.read_char();
            while let Some(c) = self.current_char {
                if c.is_digit(10) {
                    number.push(c);
                    self.read_char();
                } else {
                    break;
                }
            }
            number.parse::<f64>().ok().map(Token::Float)
        } else {
            number.parse::<i64>().ok().map(Token::Integer)
        }
    }

    fn read_string_literal(&mut self) -> Option<Token> {
        self.read_char(); // Skip opening '
        let mut string = String::new();
        while let Some(c) = self.current_char {
            if c == '\'' {
                self.read_char(); // Skip closing '
                break;
            } else {
                string.push(c);
                self.read_char();
            }
        }
        Some(Token::StringLiteral(string))
    }
}
