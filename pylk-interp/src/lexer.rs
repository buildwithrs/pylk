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
    Walrus,       // :=

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
                    if ch.is_alphabetic() {
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

        // 2. scan all digits after decimal point
        if self.peek() == Some('.') {
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
        let num = num_text
            .parse::<f64>()
            .map_err(|e| LexerError::InvalidNumber(e.to_string()))?;
        Ok(Token::Float(num))
    }

    fn parse_identifier(&mut self) -> Result<Token, LexerError> {
        while self.peek().is_some_and(|d| d.is_alphanumeric()) {
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
    use crate::lexer::{Lexer, Token};

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
}
