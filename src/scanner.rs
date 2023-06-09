use std::collections::HashMap;
use crate::token::{Token, TokenType, Literal};

//FIXME: refactor to not use a struct...is that like bad?
#[derive(Debug)]
pub struct Scanner {
    source: String,
}

impl Scanner {
    //FIXME find a better way to store this...const?
    fn keywords() -> HashMap<String, TokenType> {
        HashMap::from([
            ("and".to_string(), TokenType::AND),
            ("class".to_string(), TokenType::CLASS),
            ("else".to_string(), TokenType::ELSE),
            ("false".to_string(), TokenType::FALSE),
            ("for".to_string(), TokenType::FOR),
            ("fun".to_string(), TokenType::FUN),
            ("if".to_string(), TokenType::IF),
            ("nil".to_string(), TokenType::NIL),
            ("or".to_string(), TokenType::OR),
            ("print".to_string(), TokenType::PRINT),
            ("return".to_string(), TokenType::RETURN),
            ("super".to_string(), TokenType::SUPER),
            ("this".to_string(), TokenType::THIS),
            ("true".to_string(), TokenType::TRUE),
            ("var".to_string(), TokenType::VAR),
            ("while".to_string(), TokenType::WHILE),
        ])
    }

    #[must_use] pub const fn new(source: String) -> Self {
        Self {
            source,
        }
    }

    #[must_use] pub fn scan_tokens(&self) -> Vec<Token> {
        let chars: Vec<_> = self.source.chars().collect(); 
        let mut tokens = vec![];
        let mut current: usize = 0;
        let mut line: usize = 1;
        for _c in &chars {
            let start: usize = current;
            
            let is_at_end = current >= chars.len();
            let (value, token_type) = self.scan_token(&mut current, start, &mut line, is_at_end);

            if let Some(t) = token_type {
                match value.clone() {
                    Literal::String(s) => {
                        tokens.push(Self::create_token_literal(t, s, value, line));
                    }
                    Literal::Number(n) => {
                        tokens.push(Self::create_token_literal(t, n.to_string(), value, line));
                    }
                    Literal::None => tokens.push(Self::create_token(t, 
                                                                    self.source[start..current].to_string(), 
                                                                    line)),
                }
            } 
        }
        tokens.push(Self::create_token_literal(TokenType::EOF, 
                                              String::new(), 
                                              Literal::None, 
                                              line));
        tokens
    }

    const fn create_token(token_type: TokenType, 
                          text: String, 
                          line: usize) -> Token {
        Self::create_token_literal(token_type, text, Literal::None, line)
    }

    const fn create_token_literal(token_type: TokenType, 
                            text: String,
                            literal: Literal,
                            line: usize) -> Token {
        Token {
            token_type,
            lexeme: text,
            literal,
            line,
        } 
    }

    fn scan_token(&self, 
                  current: &mut usize,
                  start: usize,
                  line: &mut usize, 
                  is_at_end: bool) -> (Literal, Option<TokenType>) {
        let c = self.advance(current);
        match c {
            Some('(') => (Literal::None, Some(TokenType::LEFT_PAREN)),
            Some(')') => (Literal::None, Some(TokenType::RIGHT_PAREN)),
            Some('{') => (Literal::None, Some(TokenType::LEFT_BRACE)),
            Some('}') => (Literal::None, Some(TokenType::RIGHT_BRACE)),
            Some(',') => (Literal::None, Some(TokenType::COMMA)),
            Some('.') => (Literal::None, Some(TokenType::DOT)),
            Some('-') => (Literal::None, Some(TokenType::MINUS)),
            Some('+') => (Literal::None, Some(TokenType::PLUS)),
            Some(';') => (Literal::None, Some(TokenType::SEMICOLON)),
            Some('*') => (Literal::None, Some(TokenType::STAR)),
            Some('!') => {
                if self.check_next_char(is_at_end, current, '=') {
                    (Literal::None, Some(TokenType::BANG_EQUAL))
                } else {
                    (Literal::None, Some(TokenType::BANG))
                }
            }
            Some('=') => {
                if self.check_next_char(is_at_end, current, '=') {
                    (Literal::None, Some(TokenType::EQUAL_EQUAL))
                } else {
                    (Literal::None, Some(TokenType::EQUAL))
                }
            }
            Some('<') => {
                if self.check_next_char(is_at_end, current, '=') {
                    (Literal::None, Some(TokenType::LESS_EQUAL))
                } else {
                    (Literal::None, Some(TokenType::LESS))
                }
            }
            Some('>') => {
                if self.check_next_char(is_at_end, current, '=') {
                    (Literal::None, Some(TokenType::GREATER_EQUAL))
                } else {
                    (Literal::None, Some(TokenType::GREATER))
                }
            }
            Some('/') => {
                if self.check_next_char(is_at_end, current, '/') {
                    while self.peek(*current, is_at_end) != Some('\n') && !is_at_end {
                        _ = self.advance(current);
                    }
                    (Literal::None, None)
                } else if self.check_next_char(is_at_end, current, '*') {
                    while self.peek(*current, is_at_end) != Some('*') && self.peek_next(*current) != Some('/') && !is_at_end {
                        _ = self.advance(current);
                    }
                    _ = self.advance(current);
                    _ = self.advance(current);
                    (Literal::None, None)
                } else {
                    (Literal::None, Some(TokenType::SLASH))
                }
            }
            Some('"') => self.string(current, start, is_at_end, line),
            Some(' ' | '\r' | '\t') | None => (Literal::None, None),
            Some('\n') => { *line += 1; (Literal::None, None) }
            _ => {
                if Self::is_digit(c) {
                    self.number(current, is_at_end, start)
                } else if Self::is_alpha(c) {
                    self.identifier(current, is_at_end, start)
                } else {
                    eprint!("unexpected token {c:?} at line {line}");
                    (Literal::None, None)
                }
            }
        }
    }

    //stfu clippy
    #[allow(clippy::option_if_let_else)]
    fn identifier(&self, current: &mut usize, is_at_end: bool, start: usize) -> (Literal, Option<TokenType>) {
        while Self::is_alphanumeric(self.peek(*current, is_at_end)) {
            self.advance(current);
        }

        let id = &self.source[start..*current];
        let keywords = Self::keywords();
        let keyword = keywords.get(id);

        match keyword {
            Some(k) => (Literal::None, Some(k.clone())),
            None => (Literal::None, Some(TokenType::IDENTIFIER)),
        }
    }
    
    const fn is_alphanumeric(c: Option<char>) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }

    const fn is_alpha(c: Option<char>) -> bool {
        match c {
            Some(a) => {
                (a >= 'a' && a <= 'z') || (a >= 'A' && a <= 'Z') || a == '_'
            }
            None => false,
        }
    }

    const fn is_digit(c: Option<char>) -> bool {
        match c {
            Some(digit) => digit.is_ascii_digit(),
            None => false,
        }
        
    }

    fn number(&self, current: &mut usize, is_at_end: bool, start: usize) -> (Literal, Option<TokenType>) {
        while Self::is_digit(self.peek(*current, is_at_end)) {
            self.advance(current);
        }

        if self.peek(*current, is_at_end) == Some('.') && Self::is_digit(self.peek_next(*current)) {
            self.advance(current);
            while Self::is_digit(self.peek(*current, is_at_end)) {
                self.advance(current);
            }
        }

        let num = &self.source[start..*current];
        let num_val = num.parse::<f64>();
        match num_val {
            Ok(n) => (Literal::Number(n), Some(TokenType::NUMBER)),
            Err(e) => {
                eprintln!("error parsing number: {e}");
                (Literal::None, None)
            }
        }
    }

    fn string(&self, 
              current: &mut usize, 
              start: usize, 
              is_at_end: bool, 
              line: &mut usize) -> (Literal, Option<TokenType>) {
        while self.peek(*current, is_at_end) != Some('"') && !is_at_end {
            if self.peek(*current, is_at_end) == Some('\n') { *line += 1 }
            self.advance(current);
        }

        if is_at_end {
            eprint!("error line {line} unterminated string");
            return (Literal::None, None);
        }

        self.advance(current);

        let value = &self.source[start+1..*current-1];
        (Literal::String(value.to_string()), Some(TokenType::STRING))
    }

    fn advance(&self, current: &mut usize) -> Option<char> {
        let prev = *current;
        *current += 1;
        self.source.chars().nth(prev)
    }

    fn peek(&self, current: usize, is_at_end: bool) -> Option<char> {
        if is_at_end { return Some('\0') }
        self.source.chars().nth(current)
    }

    fn peek_next(&self, current: usize) -> Option<char> {
        if current + 1 >= self.source.chars().count() {
            return Some('\0');
        }
        self.source.chars().nth(current + 1)
    }

    fn check_next_char(&self,
                       is_at_end: bool, 
                       current: &mut usize, 
                       expected: char) -> bool {
        if is_at_end { return false; }
        if self.source.chars().nth(*current) != Some(expected) { return false; }

        *current += 1;
        true
    }
}

