use std::fmt::Display;

use crate::errors::LexerError;

pub const KW: &[&str] = &[
    "None", "and", "as", "break", "class", "continue", "def", "del", "elif", "else", "for", "from",
    "if", "import", "in", "is", "lambda", "not", "or", "pass", "return", "while",
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
    If,
    Import,
    In,
    Is,
    Lambda,
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

    Plus,     // +
    Minus,    // -
    Mul,      // *
    Pow,      // **
    Div,      // /
    FloorDiv, // //
    Mod,      // %
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
    LBC,      // {:
    RBC,      // :}
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

    Plus,
    Minus,
    Mul,
    Pow,
    Div,
    FloorDiv,
    Mod,
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
    LBC,
    RBC,

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

            Token::Plus => TokenType::Plus,
            Token::Minus => TokenType::Minus,
            Token::Mul => TokenType::Mul,
            Token::Pow => TokenType::Pow,
            Token::Div => TokenType::Div,
            Token::FloorDiv => TokenType::FloorDiv,
            Token::Mod => TokenType::Mod,
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
            Token::LBC => TokenType::LBC,
            Token::RBC => TokenType::RBC,
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
            match self.next_token() {
                Ok(Token::WhiteSpace) => continue,
                Ok(Token::Eof) => break,
                Err(LexerError::EOF) => break,
                Ok(tk) => tokens.push(tk),
                Err(e) => return Err(e),
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        let cur = self.advance().ok_or_else(|| LexerError::EOF)?;
        match cur {
            '\n' | '\t' | ' ' => Ok(Token::WhiteSpace),
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '[' => Ok(Token::LBracket),
            ']' => Ok(Token::RBracket),
            '{' => {
                if self.is_match(':') {
                    self.advance();
                    return Ok(Token::LBC);
                }
                Ok(Token::LBrace)
            }
            '}' => Ok(Token::RBrace),
            '.' => {
                if self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    return self.parse_number(true);
                }
                Ok(Token::Dot)
            }
            ',' => Ok(Token::Comma),
            ':' => {
                if self.is_match('}') {
                    self.advance();
                    return Ok(Token::RBC);
                }
                Ok(Token::Colon)
            }
            ';' => Ok(Token::Semi),
            '#' => {
                // skip line comment
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                Ok(Token::WhiteSpace)
            }
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
                if self.is_match('/') {
                    self.advance();
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::FloorAssign)
                    } else {
                        Ok(Token::FloorDiv)
                    }
                } else if self.is_match('=') {
                    self.advance();
                    Ok(Token::DivAssign)
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
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::ShlAssign)
                    } else {
                        Ok(Token::Shl)
                    }
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
                    if self.is_match('=') {
                        self.advance();
                        Ok(Token::ShrAssign)
                    } else {
                        Ok(Token::Shr)
                    }
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
                    return self.parse_number(false);
                } else if ch == '\'' || ch == '"' {
                    return self.parse_string(ch);
                }

                Err(LexerError::UnsupportToken(ch))
            }
        }
    }

    fn parse_string(&mut self, end: char) -> Result<Token, LexerError> {
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
    fn parse_number(&mut self, start_with_dot: bool) -> Result<Token, LexerError> {
        let mut is_float = start_with_dot;

        // 1. scan digit before decimal point
        while self.peek().is_some_and(|d| d.is_ascii_digit()) {
            self.advance();
        }

        // 2. scan all digits after decimal point
        if !start_with_dot && self.peek() == Some('.') {
            is_float = true;
            self.advance();

            while self.peek().is_some_and(|d| d.is_ascii_digit()) {
                self.advance();
            }
        }

        // 3. scan the exponent
        if matches!(self.peek(), Some('e' | 'E')) {
            is_float = true;
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
        lexer::{KW, Keyword, Lexer, Token, TokenType, assign_tokens, is_keyword},
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
                Token::Int(1),
                Token::Semi,
                Token::Ident("b".to_string()),
                Token::Assign,
                Token::Int(2),
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
                Token::Int(1),
                Token::Semi,
                Token::Ident("b".to_string()),
                Token::Assign,
                Token::Int(2),
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
                Token::Int(0),
                Token::Float(3.14),
                Token::Float(0.5),
                Token::Float(5.0),
                Token::Float(1e6),
                Token::Float(1.25e-3),
            ]
        );
    }

    #[test]
    fn test_lexer_numbers1() {
        let num = ".5";
        let n = num.parse::<f64>();
        println!("n: {:?}", n);
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
    // -----------------------------------------------------------------
    // Additional tests covering token cases not exercised above.
    // -----------------------------------------------------------------

    #[test]
    fn test_lexer_integer_literals() {
        assert_eq!(
            lex("0 7 42 12345 -3 -42").unwrap(),
            vec![
                Token::Int(0),
                Token::Int(7),
                Token::Int(42),
                Token::Int(12345),
                // `-` is its own token; the literal `-3` is `-` followed by `3`.
                Token::Minus,
                Token::Int(3),
                Token::Minus,
                Token::Int(42),
            ]
        );
    }

    #[test]
    fn test_lexer_remaining_keywords() {
        assert_eq!(
            lex("and as break class continue def del elif for from import \
                 in is lambda not or pass return")
            .unwrap(),
            vec![
                Token::Kw(Keyword::And),
                Token::Kw(Keyword::As),
                Token::Kw(Keyword::Break),
                Token::Kw(Keyword::Class),
                Token::Kw(Keyword::Continue),
                Token::Kw(Keyword::Def),
                Token::Kw(Keyword::Del),
                Token::Kw(Keyword::Elif),
                Token::Kw(Keyword::For),
                Token::Kw(Keyword::From),
                Token::Kw(Keyword::Import),
                Token::Kw(Keyword::In),
                Token::Kw(Keyword::Is),
                Token::Kw(Keyword::Lambda),
                Token::Kw(Keyword::Not),
                Token::Kw(Keyword::Or),
                Token::Kw(Keyword::Pass),
                Token::Kw(Keyword::Return),
            ]
        );
    }

    #[test]
    fn test_lexer_compound_assign_operators() {
        // Compound assignment operators that should lex as a single token.
        assert_eq!(
            lex("//= <<= >>=").unwrap(),
            vec![Token::FloorAssign, Token::ShlAssign, Token::ShrAssign,]
        );
    }

    #[test]
    fn test_lexer_skips_line_comment() {
        // Whitespace and `#` comments should be ignored between tokens.
        assert_eq!(
            lex("foo # trailing comment\nbar").unwrap(),
            vec![
                Token::Ident("foo".to_string()),
                Token::Ident("bar".to_string()),
            ]
        );
    }

    #[test]
    fn test_token_ident_method() {
        // ident() returns the underlying identifier string for Ident tokens.
        assert_eq!(
            Token::Ident("foo".to_string()).ident(),
            Some("foo".to_string())
        );
        // For any other token variant, ident() returns None.
        assert_eq!(Token::Int(1).ident(), None);
        assert_eq!(Token::Kw(Keyword::If).ident(), None);
        assert_eq!(Token::Plus.ident(), None);
        assert_eq!(Token::Str("s".to_string()).ident(), None);
    }

    #[test]
    fn test_assign_tokens_helper() {
        // assign_tokens() returns the two assignment tokens the parser
        // currently accepts (plain `=` and compound `+=`).
        assert_eq!(assign_tokens(), vec![Token::Assign, Token::PlusAssign]);
    }

    #[test]
    fn test_is_keyword_helper() {
        // Every entry in the public `KW` table is a keyword.
        for kw in KW {
            assert!(is_keyword(kw), "{} should be a keyword", kw);
        }
        // Names that look similar but are not keywords remain identifiers.
        assert!(!is_keyword("If"));
        assert!(!is_keyword("true"));
        assert!(!is_keyword("none"));
        assert!(!is_keyword("_name"));
    }

    #[test]
    fn test_keyword_from_all_variants() {
        assert_eq!(Keyword::from("and"), Keyword::And);
        assert_eq!(Keyword::from("as"), Keyword::As);
        assert_eq!(Keyword::from("break"), Keyword::Break);
        assert_eq!(Keyword::from("class"), Keyword::Class);
        assert_eq!(Keyword::from("continue"), Keyword::Continue);
        assert_eq!(Keyword::from("def"), Keyword::Def);
        assert_eq!(Keyword::from("del"), Keyword::Del);
        assert_eq!(Keyword::from("elif"), Keyword::Elif);
        assert_eq!(Keyword::from("else"), Keyword::Else);
        assert_eq!(Keyword::from("for"), Keyword::For);
        assert_eq!(Keyword::from("from"), Keyword::From);
        assert_eq!(Keyword::from("if"), Keyword::If);
        assert_eq!(Keyword::from("import"), Keyword::Import);
        assert_eq!(Keyword::from("in"), Keyword::In);
        assert_eq!(Keyword::from("is"), Keyword::Is);
        assert_eq!(Keyword::from("lambda"), Keyword::Lambda);
        assert_eq!(Keyword::from("None"), Keyword::None);
        assert_eq!(Keyword::from("not"), Keyword::Not);
        assert_eq!(Keyword::from("or"), Keyword::Or);
        assert_eq!(Keyword::from("pass"), Keyword::Pass);
        assert_eq!(Keyword::from("return"), Keyword::Return);
        assert_eq!(Keyword::from("while"), Keyword::While);
        // Unknown spellings map to Keyword::Unknown rather than panicking.
        assert_eq!(Keyword::from("nonsense"), Keyword::Unknown);
    }

    #[test]
    fn test_token_type_all_variants() {
        // Operators / delimiters / special tokens map to their unit variants.
        assert_eq!(Token::Plus.token_type(), TokenType::Plus);
        assert_eq!(Token::Minus.token_type(), TokenType::Minus);
        assert_eq!(Token::Mul.token_type(), TokenType::Mul);
        assert_eq!(Token::Pow.token_type(), TokenType::Pow);
        assert_eq!(Token::Div.token_type(), TokenType::Div);
        assert_eq!(Token::FloorDiv.token_type(), TokenType::FloorDiv);
        assert_eq!(Token::Mod.token_type(), TokenType::Mod);
        assert_eq!(Token::Shl.token_type(), TokenType::Shl);
        assert_eq!(Token::Shr.token_type(), TokenType::Shr);
        assert_eq!(Token::BitAnd.token_type(), TokenType::BitAnd);
        assert_eq!(Token::BitOr.token_type(), TokenType::BitOr);
        assert_eq!(Token::BitXor.token_type(), TokenType::BitXor);
        assert_eq!(Token::BitNot.token_type(), TokenType::BitNot);
        assert_eq!(Token::Lt.token_type(), TokenType::Lt);
        assert_eq!(Token::Le.token_type(), TokenType::Le);
        assert_eq!(Token::Gt.token_type(), TokenType::Gt);
        assert_eq!(Token::Ge.token_type(), TokenType::Ge);
        assert_eq!(Token::Eq.token_type(), TokenType::Eq);
        assert_eq!(Token::Ne.token_type(), TokenType::Ne);
        assert_eq!(Token::Assign.token_type(), TokenType::Assign);
        assert_eq!(Token::PlusAssign.token_type(), TokenType::PlusAssign);
        assert_eq!(Token::MinusAssign.token_type(), TokenType::MinusAssign);
        assert_eq!(Token::MulAssign.token_type(), TokenType::MulAssign);
        assert_eq!(Token::DivAssign.token_type(), TokenType::DivAssign);
        assert_eq!(Token::FloorAssign.token_type(), TokenType::FloorAssign);
        assert_eq!(Token::ModAssign.token_type(), TokenType::ModAssign);
        assert_eq!(Token::PowAssign.token_type(), TokenType::PowAssign);
        assert_eq!(Token::BitAndAssign.token_type(), TokenType::BitAndAssign);
        assert_eq!(Token::BitOrAssign.token_type(), TokenType::BitOrAssign);
        assert_eq!(Token::BitXorAssign.token_type(), TokenType::BitXorAssign);
        assert_eq!(Token::ShlAssign.token_type(), TokenType::ShlAssign);
        assert_eq!(Token::ShrAssign.token_type(), TokenType::ShrAssign);
        assert_eq!(Token::LParen.token_type(), TokenType::LParen);
        assert_eq!(Token::RParen.token_type(), TokenType::RParen);
        assert_eq!(Token::LBracket.token_type(), TokenType::LBracket);
        assert_eq!(Token::RBracket.token_type(), TokenType::RBracket);
        assert_eq!(Token::LBrace.token_type(), TokenType::LBrace);
        assert_eq!(Token::RBrace.token_type(), TokenType::RBrace);
        assert_eq!(Token::Comma.token_type(), TokenType::Comma);
        assert_eq!(Token::Colon.token_type(), TokenType::Colon);
        assert_eq!(Token::Dot.token_type(), TokenType::Dot);
        assert_eq!(Token::Semi.token_type(), TokenType::Semi);
        assert_eq!(Token::Arrow.token_type(), TokenType::Arrow);
        // Payloads erase to the kind-only variant.
        assert_eq!(Token::Kw(Keyword::If).token_type(), TokenType::Kw,);
        assert_eq!(
            Token::Ident("foo".to_string()).token_type(),
            TokenType::Ident,
        );
        assert_eq!(Token::Int(42).token_type(), TokenType::Int);
        assert_eq!(Token::Float(1.5).token_type(), TokenType::Float);
        assert_eq!(Token::Bool(true).token_type(), TokenType::Bool);
        assert_eq!(Token::Str("s".to_string()).token_type(), TokenType::Str,);
        assert_eq!(
            Token::Error("boom".to_string()).token_type(),
            TokenType::Error,
        );
    }

    #[test]
    fn test_token_display_round_trip() {
        // Each Token variant should produce a stable Display string via
        // the Debug formatter; this guards accidental Display overrides.
        let cases = vec![
            Token::Kw(Keyword::If),
            Token::Ident("foo".to_string()),
            Token::Int(42),
            Token::Float(1.5),
            Token::Bool(true),
            Token::Str("hi".to_string()),
            Token::Plus,
            Token::Minus,
            Token::Mul,
            Token::Pow,
            Token::Div,
            Token::FloorDiv,
            Token::Mod,
            Token::Shl,
            Token::Shr,
            Token::BitAnd,
            Token::BitOr,
            Token::BitXor,
            Token::BitNot,
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
            Token::FloorAssign,
            Token::ModAssign,
            Token::PowAssign,
            Token::BitAndAssign,
            Token::BitOrAssign,
            Token::BitXorAssign,
            Token::ShlAssign,
            Token::ShrAssign,
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
            Token::Eof,
            Token::WhiteSpace,
            Token::Error("boom".to_string()),
        ];
        for tk in cases {
            // Just ensure Display is implemented and yields a non-empty string.
            let s = format!("{}", tk);
            assert!(!s.is_empty(), "Display for {:?} was empty", tk);
        }
    }

    #[test]
    fn test_token_type_display() {
        // TokenType also implements Display via the same Debug formatter.
        for tt in [
            TokenType::Kw,
            TokenType::Ident,
            TokenType::Int,
            TokenType::Float,
            TokenType::Bool,
            TokenType::Str,
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Mul,
            TokenType::Pow,
            TokenType::Div,
            TokenType::FloorDiv,
            TokenType::Mod,
            TokenType::Shl,
            TokenType::Shr,
            TokenType::BitAnd,
            TokenType::BitOr,
            TokenType::BitXor,
            TokenType::BitNot,
            TokenType::Lt,
            TokenType::Le,
            TokenType::Gt,
            TokenType::Ge,
            TokenType::Eq,
            TokenType::Ne,
            TokenType::Assign,
            TokenType::PlusAssign,
            TokenType::MinusAssign,
            TokenType::MulAssign,
            TokenType::DivAssign,
            TokenType::FloorAssign,
            TokenType::ModAssign,
            TokenType::PowAssign,
            TokenType::BitAndAssign,
            TokenType::BitOrAssign,
            TokenType::BitXorAssign,
            TokenType::ShlAssign,
            TokenType::ShrAssign,
            TokenType::LParen,
            TokenType::RParen,
            TokenType::LBracket,
            TokenType::RBracket,
            TokenType::LBrace,
            TokenType::RBrace,
            TokenType::Comma,
            TokenType::Colon,
            TokenType::Dot,
            TokenType::Semi,
            TokenType::Arrow,
            TokenType::Eof,
            TokenType::WhiteSpace,
            TokenType::Error,
        ] {
            let s = format!("{}", tt);
            assert!(!s.is_empty(), "Display for {:?} was empty", tt);
        }
    }

    #[test]
    fn test_dict_literal() {
        let res = lex("{: 'x': 123, 'y': 456 :}");
        assert!(res.is_ok());
        assert_eq!(
            vec![
                Token::LBC,
                Token::Str('x'.to_string()),
                Token::Colon,
                Token::Int(123),
                Token::Comma,
                Token::Str('y'.to_string()),
                Token::Colon,
                Token::Int(456),
                Token::RBC
            ],
            res.unwrap()
        );
    }
}
