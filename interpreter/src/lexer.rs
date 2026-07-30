use std::fmt::Display;

use crate::errors::LexerError;

pub const KW: &[&str] = &[
    "None", "and", "as", "break", "class", "continue", "def", "del", "elif", "else", "for", "from",
    "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "return",
    "while",
];

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    As,
    Break,
    Class,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Is,
    Lambda,
    Nonlocal,
    None,
    And,
    Not,
    Or,
    Pass,
    Return,
    While,
    Unknown,
}

impl From<&str> for Keyword {
    fn from(value: &str) -> Self {
        match value {
            "as" => Self::As,
            "break" => Self::Break,
            "class" => Self::Class,
            "continue" => Self::Continue,
            "def" => Self::Def,
            "del" => Self::Del,
            "elif" => Self::Elif,
            "else" => Self::Else,
            "if" => Self::If,
            "for" => Self::For,
            "while" => Self::While,
            "in" => Self::In,
            "is" => Self::Is,
            "and" => Self::And,
            "or" => Self::Or,
            "not" => Self::Not,
            "return" => Self::Return,
            "import" => Self::Import,
            "from" => Self::From,
            "lambda" => Self::Lambda,
            "global" => Self::Global,
            "nonlocal" => Self::Nonlocal,
            "None" => Self::None,
            "pass" => Self::Pass,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Kw(Keyword),
    Ident(String),
    Int(i64),
    Float(f64),
    Bool(bool), // True, False
    Str(String),
    Ellipsis,

    Plus,     // +
    Minus,    // -
    Mul,      // *
    Pow,      // **
    Div,      // /
    FloorDiv, // //
    Mod,      // %
    MatMul,   // @
    Shl,      // <<
    Shr,      // >>
    BitAnd,   // &
    BitOr,    // |
    BitXor,   // ^
    BitNot,   // ~

    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=
    Eq, // ==
    Ne, // !=

    Assign, // =
    /// +=
    PlusAssign,
    MinusAssign,  // -=
    MulAssign,    // *=
    DivAssign,    // /=
    FloorAssign,  // //=
    ModAssign,    // %=
    PowAssign,    // **=
    MatMulAssign, // @=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    ShlAssign,    // <<=
    ShrAssign,    // >>=

    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }
    Comma,    // ,
    Colon,    // :
    Dot,      // .
    Semi,     // ;
    Arrow,    // ->

    Eof,
    WhiteSpace,
    Error(String),
}

/// Discriminant of [`Token`] without its associated data.
///
/// Lets parser code match on the kind of a token without needing to
/// destructure or care about the value it carries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum TokenType {
    Kw,
    Ident,
    Int,
    Float,
    Bool,
    Str,
    Ellipsis,

    Plus,
    Minus,
    Mul,
    Pow,
    Div,
    FloorDiv,
    Mod,
    MatMul,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,

    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,

    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    FloorAssign,
    ModAssign,
    PowAssign,
    MatMulAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Dot,
    Semi,
    Arrow,

    Eof,
    WhiteSpace,
    Error,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn assign_tokens() -> Vec<Token> {
    vec![Token::Assign, Token::PlusAssign]
}

impl Token {
    pub fn ident(&self) -> Option<String> {
        if let Token::Ident(id) = self {
            return Some(id.clone());
        }
        None
    }

    /// Returns the [`TokenType`] (kind) of this token.
    pub fn token_type(&self) -> TokenType {
        match self {
            Token::Kw(_) => TokenType::Kw,
            Token::Ident(_) => TokenType::Ident,
            Token::Int(_) => TokenType::Int,
            Token::Float(_) => TokenType::Float,
            Token::Bool(_) => TokenType::Bool,
            Token::Str(_) => TokenType::Str,
            Token::Ellipsis => TokenType::Ellipsis,

            Token::Plus => TokenType::Plus,
            Token::Minus => TokenType::Minus,
            Token::Mul => TokenType::Mul,
            Token::Pow => TokenType::Pow,
            Token::Div => TokenType::Div,
            Token::FloorDiv => TokenType::FloorDiv,
            Token::Mod => TokenType::Mod,
            Token::MatMul => TokenType::MatMul,
            Token::Shl => TokenType::Shl,
            Token::Shr => TokenType::Shr,
            Token::BitAnd => TokenType::BitAnd,
            Token::BitOr => TokenType::BitOr,
            Token::BitXor => TokenType::BitXor,
            Token::BitNot => TokenType::BitNot,

            Token::Lt => TokenType::Lt,
            Token::Le => TokenType::Le,
            Token::Gt => TokenType::Gt,
            Token::Ge => TokenType::Ge,
            Token::Eq => TokenType::Eq,
            Token::Ne => TokenType::Ne,

            Token::Assign => TokenType::Assign,
            Token::PlusAssign => TokenType::PlusAssign,
            Token::MinusAssign => TokenType::MinusAssign,
            Token::MulAssign => TokenType::MulAssign,
            Token::DivAssign => TokenType::DivAssign,
            Token::FloorAssign => TokenType::FloorAssign,
            Token::ModAssign => TokenType::ModAssign,
            Token::PowAssign => TokenType::PowAssign,
            Token::MatMulAssign => TokenType::MatMulAssign,
            Token::BitAndAssign => TokenType::BitAndAssign,
            Token::BitOrAssign => TokenType::BitOrAssign,
            Token::BitXorAssign => TokenType::BitXorAssign,
            Token::ShlAssign => TokenType::ShlAssign,
            Token::ShrAssign => TokenType::ShrAssign,

            Token::LParen => TokenType::LParen,
            Token::RParen => TokenType::RParen,
            Token::LBracket => TokenType::LBracket,
            Token::RBracket => TokenType::RBracket,
            Token::LBrace => TokenType::LBrace,
            Token::RBrace => TokenType::RBrace,
            Token::Comma => TokenType::Comma,
            Token::Colon => TokenType::Colon,
            Token::Dot => TokenType::Dot,
            Token::Semi => TokenType::Semi,
            Token::Arrow => TokenType::Arrow,

            Token::Eof => TokenType::Eof,
            Token::WhiteSpace => TokenType::WhiteSpace,
            Token::Error(_) => TokenType::Error,
        }
    }
}

pub struct Lexer {
    pub tokens: Vec<Token>,
    pub chars: Vec<char>,
    pub start: usize,
    pub current: usize,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Self {
            tokens: vec![],
            chars: src.to_string().chars().collect(),
            start: 0,
            current: 0,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens: Vec<Token> = Vec::with_capacity(self.chars.len());
        loop {
            if self.is_end() {
                break;
            }

            self.start = self.current;
            let tk = self.next_token()?;
            if tk == Token::WhiteSpace {
                continue;
            }

            tokens.push(tk);
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        if let Some(cur) = self.advance() {
            match cur {
                '\n' | '\t' | ' ' => Ok(Token::WhiteSpace),
                '(' => Ok(Token::LParen),
                ')' => Ok(Token::RParen),
                '[' => Ok(Token::LBracket),
                ']' => Ok(Token::RBracket),
                '{' => Ok(Token::LBrace),
                '}' => Ok(Token::RBrace),
                '.' => {
                    if let Some(ch) = self.peek() {
                        if ch.is_ascii_digit() {
                            return self.parse_number();
                        }
                    }
                    return Ok(Token::Dot);
                }
                ',' => Ok(Token::Comma),
                ':' => Ok(Token::Colon),
                ';' => Ok(Token::Semi),
                '&' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::BitAndAssign)
                    } else {
                        Ok(Token::BitAnd)
                    }
                }
                '|' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::BitOrAssign)
                    } else {
                        Ok(Token::BitOr)
                    }
                }
                '^' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::BitXorAssign)
                    } else {
                        Ok(Token::BitXor)
                    }
                }
                '~' => Ok(Token::BitNot),
                '+' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::PlusAssign)
                    } else {
                        Ok(Token::Plus)
                    }
                }
                '-' => {
                    if self.is_match('>') {
                        self.advance();
                        Ok(Token::Arrow)
                    } else if self.is_match('=') {
                        self.advance();
                        Ok(Token::MinusAssign)
                    } else {
                        Ok(Token::Minus)
                    }
                }
                '*' => {
                    if self.is_match('*') {
                        self.advance();
                        if self.is_match('=') {
                            self.advance();
                            Ok(Token::PowAssign)
                        } else {
                            Ok(Token::Pow)
                        }
                    } else if self.is_match('=') {
                        self.advance();
                        Ok(Token::MulAssign)
                    } else {
                        Ok(Token::Mul)
                    }
                }
                '/' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::DivAssign)
                    } else if self.is_match('/') {
                        self.advance();
                        Ok(Token::FloorDiv)
                    } else {
                        Ok(Token::Div)
                    }
                }
                '%' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::ModAssign)
                    } else {
                        Ok(Token::Mod)
                    }
                }
                '=' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::Eq)
                    } else {
                        Ok(Token::Assign)
                    }
                }
                '<' => {
                    if self.is_match('<') {
                        self.advance();
                        Ok(Token::Shl)
                    } else if self.is_match('=') {
                        self.advance();
                        Ok(Token::Le)
                    } else {
                        Ok(Token::Lt)
                    }
                }
                '>' => {
                    if self.is_match('>') {
                        self.advance();
                        Ok(Token::Shr)
                    } else if self.is_match('=') {
                        self.advance();
                        Ok(Token::Ge)
                    } else {
                        Ok(Token::Gt)
                    }
                }
                '!' => {
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::Ne)
                    } else {
                        Err(LexerError::InvalidToken(format!("`!`")))
                    }
                }
                ch => {
                    if ch.is_alphabetic() || ch.eq(&'_') {
                        return self.parse_identifier();
                    } else if ch.is_digit(10) {
                        return self.parse_number();
                    } else if ch == '\'' || ch == '"' {
                        return self.parse_string(ch);
                    }

                    Err(LexerError::UnsupportToken(ch))
                }
            }
        } else {
            Ok(Token::Eof)
        }
    }

    fn parse_string(&mut self, end: char) -> Result<Token, LexerError> {
        println!(
            "parsing string at pos: start({}), current({})",
            self.start, self.current
        );
        let mut s_end = false;
        while let Some(ch) = self.peek() {
            if ch != end {
                self.advance();
                continue;
            }

            s_end = true;
            break;
        }

        if self.is_end() && !s_end {
            return Err(LexerError::InvalidString("unterminated string".to_string()));
        }

        let start = self.start + 1; // skip start ' or "
        let s = String::from_iter(&self.chars[start..self.current]);
        self.advance(); // skip end
        Ok(Token::Str(s))
    }

    ///
    /*
    3.14
    0.5
    .5
    5.
    1e6
    1.25e-3
    1_000.25
    */
    fn parse_number(&mut self) -> Result<Token, LexerError> {
        // 1. scan digit before decimal point
        while self.peek().is_some_and(|d| d.is_ascii_digit()) {
            self.advance();
        }

        let mut is_float = false;

        // 2. scan all digits after decimal point
        if self.peek() == Some('.') {
            is_float = true;
            self.advance();

            while self.peek().is_some_and(|d| d.is_ascii_digit()) {
                self.advance();
            }
        }

        // 3. scan the exponent
        if matches!(self.peek(), Some('e' | 'E')) {
            self.advance();

            if matches!(self.peek(), Some('+' | '-')) {
                self.advance();
            }

            let exp_start = self.current;
            while self.peek().is_some_and(|d| d.is_ascii_digit()) {
                self.advance();
            }

            if exp_start == self.current {
                return Err(LexerError::InvalidNumber(format!(
                    "invalid number: no digit after exponent."
                )));
            }
        }

        let num_text = String::from_iter(&self.chars[self.start..self.current]);
        if is_float {
            let num = num_text
                .parse::<f64>()
                .map_err(|e| LexerError::InvalidNumber(e.to_string()))?;
            return Ok(Token::Float(num));
        }

        let num = num_text
            .parse::<i64>()
            .map_err(|e| LexerError::InvalidNumber(e.to_string()))?;
        Ok(Token::Int(num))
    }

    fn parse_identifier(&mut self) -> Result<Token, LexerError> {
        while self
            .peek()
            .is_some_and(|d| d.is_alphanumeric() || d.eq(&'_'))
        {
            self.advance();
        }

        let ident = String::from_iter(&self.chars[self.start..self.current]);

        match ident.as_str() {
            "True" => Ok(Token::Bool(true)),
            "False" => Ok(Token::Bool(false)),
            _ => {
                if is_keyword(&ident) {
                    return Ok(Token::Kw(Keyword::from(ident.as_ref())));
                } else {
                    return Ok(Token::Ident(ident));
                }
            }
        }
    }

    fn is_match(&self, ch: char) -> bool {
        if let Some(next) = self.peek() {
            ch == next
        } else {
            false
        }
    }

    fn is_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            Some(self.chars[self.current])
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.is_end() || self.current + 1 >= self.chars.len() {
            None
        } else {
            Some(self.chars[self.current + 1])
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_end() {
            None
        } else {
            self.current += 1;
            Some(self.chars[self.current - 1])
        }
    }
}

pub fn is_keyword(ident: &str) -> bool {
    KW.contains(&ident)
}

#[cfg(test)]
mod tests {
    use crate::{
        errors::LexerError,
        lexer::{Keyword, Lexer, Token, TokenType},
    };

    fn lex(source: &str) -> Result<Vec<Token>, LexerError> {
        Lexer::new(source).lex()
    }

    #[test]
    fn test_lexer_simple() {
        let source = r#"a = 1;
        b = 2;
        c = a + b;"#;
        let mut lexer = Lexer::new(source);
        let tokens_result = lexer.lex();
        assert!(tokens_result.is_ok());

        let tokens = tokens_result.unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".to_string()),
                Token::Assign,
                Token::Float(1.0),
                Token::Semi,
                Token::Ident("b".to_string()),
                Token::Assign,
                Token::Float(2.0),
                Token::Semi,
                Token::Ident("c".to_string()),
                Token::Assign,
                Token::Ident("a".to_string()),
                Token::Plus,
                Token::Ident("b".to_string()),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn test_lexer_string() {
        let source = r#"a = 1;
        b = 2;
        c = "Hello";"#;

        println!("source(34): {:?}", source.chars().nth(34));
        println!("source(35): {:?}", source.chars().nth(35));

        let mut lexer = Lexer::new(source);
        let tokens_result = lexer.lex();
        assert!(tokens_result.is_ok());

        let tokens = tokens_result.unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".to_string()),
                Token::Assign,
                Token::Float(1.0),
                Token::Semi,
                Token::Ident("b".to_string()),
                Token::Assign,
                Token::Float(2.0),
                Token::Semi,
                Token::Ident("c".to_string()),
                Token::Assign,
                Token::Str("Hello".to_string()),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn test_lexer_class() {
        let source = r#"class Person {
            def __init__(self, name, age) {
                self.name = name;
                self.age = age;
            }
        }"#;

        let mut lexer = Lexer::new(source);
        let tokens_result = lexer.lex();
        println!("tokens_result: {:?}", tokens_result);

        assert!(tokens_result.is_ok());
        let tokens = tokens_result.unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Kw(Keyword::Class),
                Token::Ident("Person".to_string()),
                Token::LBrace,
                Token::Kw(Keyword::Def),
                Token::Ident("__init__".to_string()),
                Token::LParen,
                Token::Ident("self".to_string()),
                Token::Comma,
                Token::Ident("name".to_string()),
                Token::Comma,
                Token::Ident("age".to_string()),
                Token::RParen,
                Token::LBrace,
                Token::Ident("self".to_string()),
                Token::Dot,
                Token::Ident("name".to_string()),
                Token::Assign,
                Token::Ident("name".to_string()),
                Token::Semi,
                Token::Ident("self".to_string()),
                Token::Dot,
                Token::Ident("age".to_string()),
                Token::Assign,
                Token::Ident("age".to_string()),
                Token::Semi,
                Token::RBrace,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn test_lexer_operator_families() {
        assert_eq!(
            lex("+ - * ** / // % & | ^ ~ << >> < <= > >= == != = += -= *= /= %= **= &= |= ^=")
                .unwrap(),
            vec![
                Token::Plus,
                Token::Minus,
                Token::Mul,
                Token::Pow,
                Token::Div,
                Token::FloorDiv,
                Token::Mod,
                Token::BitAnd,
                Token::BitOr,
                Token::BitXor,
                Token::BitNot,
                Token::Shl,
                Token::Shr,
                Token::Lt,
                Token::Le,
                Token::Gt,
                Token::Ge,
                Token::Eq,
                Token::Ne,
                Token::Assign,
                Token::PlusAssign,
                Token::MinusAssign,
                Token::MulAssign,
                Token::DivAssign,
                Token::ModAssign,
                Token::PowAssign,
                Token::BitAndAssign,
                Token::BitOrAssign,
                Token::BitXorAssign,
            ]
        );
    }

    #[test]
    fn test_lexer_delimiters_and_arrow() {
        assert_eq!(
            lex("()[]{} , : . ; ->").unwrap(),
            vec![
                Token::LParen,
                Token::RParen,
                Token::LBracket,
                Token::RBracket,
                Token::LBrace,
                Token::RBrace,
                Token::Comma,
                Token::Colon,
                Token::Dot,
                Token::Semi,
                Token::Arrow,
            ]
        );
    }

    #[test]
    fn test_lexer_keywords_booleans_and_identifiers() {
        assert_eq!(
            lex("if else while True False None _name className").unwrap(),
            vec![
                Token::Kw(Keyword::If),
                Token::Kw(Keyword::Else),
                Token::Kw(Keyword::While),
                Token::Bool(true),
                Token::Bool(false),
                Token::Kw(Keyword::None),
                Token::Ident("_name".to_string()),
                Token::Ident("className".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_numbers() {
        assert_eq!(
            lex("0 3.14 .5 5. 1e6 1.25e-3").unwrap(),
            vec![
                Token::Float(0.0),
                Token::Float(3.14),
                Token::Float(0.5),
                Token::Float(5.0),
                Token::Float(1e6),
                Token::Float(1.25e-3),
            ]
        );
    }

    #[test]
    fn test_lexer_empty_and_single_quoted_strings() {
        assert_eq!(
            lex(r#""" '' 'hello'"#).unwrap(),
            vec![
                Token::Str(String::new()),
                Token::Str(String::new()),
                Token::Str("hello".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_skips_whitespace() {
        assert_eq!(
            lex("\n\t foo \n bar\t").unwrap(),
            vec![
                Token::Ident("foo".to_string()),
                Token::Ident("bar".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_rejects_unterminated_string() {
        assert!(matches!(
            lex("\"unterminated"),
            Err(LexerError::InvalidString(_))
        ));
    }

    #[test]
    fn test_lexer_rejects_invalid_exponent() {
        assert!(matches!(lex("1e+"), Err(LexerError::InvalidNumber(_))));
    }

    #[test]
    fn test_lexer_rejects_standalone_bang() {
        assert!(matches!(lex("!"), Err(LexerError::InvalidToken(_))));
    }

    #[test]
    fn test_lexer_rejects_unsupported_character() {
        assert!(matches!(lex("@"), Err(LexerError::UnsupportToken('@'))));
    }

    #[test]
    fn test_token_type() {
        // Discardable tokens map to the unit-style variants.
        assert_eq!(Token::Plus.token_type(), TokenType::Plus);
        assert_eq!(Token::Minus.token_type(), TokenType::Minus);
        assert_eq!(Token::Eof.token_type(), TokenType::Eof);
        assert_eq!(Token::WhiteSpace.token_type(), TokenType::WhiteSpace);

        // Tokens with payloads map to the kind-only variant.
        assert_eq!(
            Token::Ident("foo".to_string()).token_type(),
            TokenType::Ident
        );
        assert_eq!(Token::Int(42).token_type(), TokenType::Int);
        assert_eq!(Token::Float(1.5).token_type(), TokenType::Float);
        assert_eq!(Token::Bool(true).token_type(), TokenType::Bool);
        assert_eq!(Token::Str("s".to_string()).token_type(), TokenType::Str);
        assert_eq!(Token::Kw(Keyword::If).token_type(), TokenType::Kw,);
        assert_eq!(
            Token::Error("boom".to_string()).token_type(),
            TokenType::Error,
        );
    }
}
