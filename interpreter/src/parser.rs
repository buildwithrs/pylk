use crate::{
    ast::{AssignOperator, AssignTarget, Block, Expr, ImportName, LiteralExpr, Node, Parameter, Program, Stmt}, errors::ParserError, lexer::{Keyword, Token, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Parser {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut nodes = Vec::new();

        loop {
            while self.check(&Token::Semi) {
                self.advance();
            }

            if self.is_end() {
                break;
            }

            let stmt = self.parse_stmt()?;
            nodes.push(Node::new(stmt));
        }

        Ok(Program(nodes))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        if let Some(cur) = self.advance() {
            match cur {
                Token::Kw(Keyword::Import) => self.parse_import(),
                Token::Kw(Keyword::From) => self.parse_from_import(),
                Token::Kw(Keyword::Class) => self.parse_class(),
                Token::Kw(Keyword::Def) => self.parse_function(),
                Token::Kw(Keyword::If) => self.parse_if(),
                Token::Kw(Keyword::While) => self.parse_while(),
                Token::Kw(Keyword::For) => self.parse_for(),
                Token::Kw(Keyword::Return) => self.parse_return(),
                Token::Kw(Keyword::Pass) => Ok(Stmt::Pass),
                Token::Kw(Keyword::Break) => Ok(Stmt::Break),
                Token::Kw(Keyword::Continue) => Ok(Stmt::Continue),
                _ => {
                    let exp = self.parse_expression(cur)?;
                    Ok(Stmt::ExprStmt(Box::new(exp)))
                }
            }
        } else {
            Err(ParserError::EOF)
        }
    }

    /// Parses `import a.b.c [as alias]`. The leading `import` token
    /// has already been consumed by `parse_stmt`.
    fn parse_import(&mut self) -> Result<Stmt, ParserError> {
        let path = self.parse_dotted_name()?;

        let alias = if self.check(&Token::Kw(Keyword::As)) {
            self.advance(); // consume `as`
            Some(self.expect_ident()?)
        } else {
            None
        };

        Ok(Stmt::Import { path, alias })
    }

    /// Parses `from a.b.c import name1, name2 as alias, *`.
    /// The leading `from` token has already been consumed by `parse_stmt`.
    fn parse_from_import(&mut self) -> Result<Stmt, ParserError> {
        let module = self.parse_dotted_name()?;

        let import_kw = self.advance();
        match import_kw {
            Some(Token::Kw(Keyword::Import)) => {}
            Some(other) => return Err(ParserError::UnsupportToken(other)),
            None => return Err(ParserError::EOF),
        }

        let names = self.parse_import_names()?;
        Ok(Stmt::FromImport { module, names })
    }

    /// Parses `identifier ("." identifier)*`.
    fn parse_dotted_name(&mut self) -> Result<Vec<String>, ParserError> {
        let mut parts = vec![self.expect_ident()?];
        while self.check(&Token::Dot) {
            self.advance(); // consume `.`
            parts.push(self.expect_ident()?);
        }
        Ok(parts)
    }

    /// Parses the import target list after `import`:
    /// `*` | `import_name ("," import_name)*`.
    fn parse_import_names(&mut self) -> Result<Vec<ImportName>, ParserError> {
        // Star form: `from x import *`
        if self.check(&Token::Mul) {
            self.advance();
            return Ok(vec![ImportName {
                name: "*".to_string(),
                alias: None,
            }]);
        }

        // Named form: at least one entry, optionally followed by more.
        let mut names = vec![self.parse_one_import_name()?];
        while self.check(&Token::Comma) {
            self.advance(); // consume `,`
            names.push(self.parse_one_import_name()?);
        }
        Ok(names)
    }

    /// Parses `identifier [as identifier]`.
    fn parse_one_import_name(&mut self) -> Result<ImportName, ParserError> {
        let name = self.expect_ident()?;
        let alias = if self.check(&Token::Kw(Keyword::As)) {
            self.advance(); // consume `as`
            Some(self.expect_ident()?)
        } else {
            None
        };
        Ok(ImportName { name, alias })
    }

    /// Consumes the next token; errors if it is not an identifier.
    fn expect_ident(&mut self) -> Result<String, ParserError> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s),
            Some(other) => Err(ParserError::UnsupportToken(other)),
            None => Err(ParserError::EOF),
        }
    }

    /// True if the next token equals `t` (without consuming it).
    fn check(&self, t: &Token) -> bool {
        if self.is_end() {
            false
        } else {
            &self.tokens[self.current] == t
        }
    }

    fn parse_class(&mut self) -> Result<Stmt, ParserError> {
        let class_name = self.expect_ident()?;
        let _ = self.consume(TokenType::LBrace)?;


        let mut members= Vec::new();
        while !self.check(&Token::RBrace) {
            let next = self.advance();
            let member = match next {
                Some(nt) => match nt {
                    Token::Kw(Keyword::Def) => self.parse_function(),
                    _ => Ok(Stmt::ExprStmt(Box::new(self.parse_expression(nt)?)))
                }

                None => Err(ParserError::EOF)
            };

            members.push(member?);
        }

        Ok(Stmt::Class { name: class_name, members: members })
    }

    fn parse_function(&mut self) -> Result<Stmt, ParserError> {
        let fn_name = self.expect_ident()?;
        let _ = self.consume(TokenType::LParen)?;
        
        // get arguments
        let mut args = Vec::new();
        while self.check(&Token::Comma) {
            self.advance();

            args.push(Parameter {
                name: self.expect_ident()?,
                default: None,
            });
        }
        self.consume(TokenType::RParen)?;

        let body = self.parse_block()?;
        Ok(Stmt::Func { name: fn_name, param_list: args, body: body })
    }

    fn parse_block(&mut self) -> Result<Block, ParserError> {
        self.consume(TokenType::LBrace)?;

        let mut stmts = Vec::new();
        while self.check(&Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.advance();

        Ok(Block(stmts))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParserError> {
        todo!()
    }

    fn parse_while(&mut self) -> Result<Stmt, ParserError> {
        todo!()
    }

    fn parse_for(&mut self) -> Result<Stmt, ParserError> {
        todo!()
    }

    fn parse_return(&mut self) -> Result<Stmt, ParserError> {
        todo!()
    }

    fn parse_expression(&mut self, t: Token) -> Result<Expr, ParserError> {
        match t {
            Token::Kw(Keyword::Lambda) => self.parse_lambda(),
            _ => self.parse_assignment(t),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParserError> {
        todo!()
    }

    fn parse_assignment(&mut self, t: Token) -> Result<Expr, ParserError> {
        // x = 123
        if let Token::Ident(x) = t {
            let _ = self.consume_oneof_types(&[TokenType::Assign])?;
            let cur = self.advance();
            if cur.is_none() {
                return Err(ParserError::EOF);
            }

            let value = self.parse_expression(cur.unwrap())?;
            Ok(Expr::Assign {
                target: AssignTarget::Name(x),
                op: AssignOperator::Assign,
                expr: Box::new(value),
            })
        } else {
            self.parse_literal(t)
        }
    }

    fn parse_literal(&mut self, t: Token) -> Result<Expr, ParserError> {
        match t {
            Token::Str(s) => Ok(Expr::Literal(LiteralExpr::Str(s))),
            Token::Int(i) => Ok(Expr::Literal(LiteralExpr::Int(i))),
            Token::Float(f) => Ok(Expr::Literal(LiteralExpr::Float(f))),
            Token::Bool(b) => Ok(Expr::Literal(LiteralExpr::Boolean(b))),
            Token::Kw(Keyword::None) => Ok(Expr::Literal(LiteralExpr::None)),
            _ => Err(ParserError::UnsupportToken(t)),
        }
    }

    fn consume(&mut self, token_type: TokenType) -> Result<Token, ParserError> {
        let t = self.advance();
        if t.is_none() {
            return Err(ParserError::EOF);
        }

        let tt = t.unwrap();
        if tt.token_type() != token_type {
            return Err(ParserError::ExpectToken(token_type, tt.token_type()));
        }

        Ok(tt)
    }

    fn consume_oneof_types(&mut self, token_types: &[TokenType]) -> Result<Token, ParserError> {
        let t = self.advance();
        if t.is_none() {
            return Err(ParserError::EOF);
        }

        let tt = t.unwrap();
        if !token_types.contains(&tt.token_type()) {
            return Err(ParserError::ExpectToken(token_types[0], tt.token_type()));
        }

        Ok(tt)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            self.current += 1;
            Some(self.tokens[self.current - 1].clone())
        }
    }

    fn is_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{AssignOperator, AssignTarget, Expr, ImportName, LiteralExpr, Node, Program, Stmt},
        lexer::Lexer,
        parser::Parser,
    };

    #[test]
    fn test_parse_literal() {
        let source = r#" 1.001 "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex();
        assert!(tokens.is_ok());
        let ts = tokens.unwrap();

        let mut p = Parser::new(ts);
        let tree = p.parse();
        assert!(tree.is_ok());

        let program = tree.unwrap();
        println!("program: {:?}", program);

        let nodes = vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Literal(
            LiteralExpr::Float(1.001),
        ))))];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_literal1() {
        let source = r#" "Hello, World" "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex();
        assert!(tokens.is_ok());
        let ts = tokens.unwrap();

        let mut p = Parser::new(ts);
        let tree = p.parse();
        assert!(tree.is_ok());

        let program = tree.unwrap();
        println!("program: {:?}", program);

        let nodes = vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Literal(
            LiteralExpr::Str("Hello, World".to_string()),
        ))))];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_assign() {
        let source = r#" x=123 "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex();
        assert!(tokens.is_ok());
        let ts = tokens.unwrap();

        let mut p = Parser::new(ts);
        let tree = p.parse();
        println!("result: {:?}", tree);
        assert!(tree.is_ok());

        let program = tree.unwrap();
        println!("program: {:?}", program);

        let nodes = vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Assign {
            target: AssignTarget::Name("x".to_string()),
            op: AssignOperator::Assign,
            expr: Box::new(Expr::Literal(LiteralExpr::Float(123.0))),
        })))];
        assert_eq!(program, Program(nodes));
    }

    /// Helper: lex + parse, returning the resulting Program.
    /// Panics on lexer/parser failure — tests use it only on inputs
    /// known to be valid.
    fn parse_source(source: &str) -> Program {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().expect("lex failed");
        let mut p = Parser::new(tokens);
        p.parse().expect("parse failed")
    }

    #[test]
    fn test_parse_import_simple() {
        let program = parse_source("import foo;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Import {
                path: vec!["foo".to_string()],
                alias: None,
            })])
        );
    }

    #[test]
    fn test_parse_import_dotted() {
        let program = parse_source("import a.b.c;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Import {
                path: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                alias: None,
            })])
        );
    }

    #[test]
    fn test_parse_import_with_alias() {
        let program = parse_source("import numpy as np;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Import {
                path: vec!["numpy".to_string()],
                alias: Some("np".to_string()),
            })])
        );
    }

    #[test]
    fn test_parse_from_import_single() {
        let program = parse_source("from os import path;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::FromImport {
                module: vec!["os".to_string()],
                names: vec![ImportName {
                    name: "path".to_string(),
                    alias: None,
                }],
            })])
        );
    }

    #[test]
    fn test_parse_from_import_multiple_with_aliases() {
        let program = parse_source("from os import path, getcwd as cwd, sep;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::FromImport {
                module: vec!["os".to_string()],
                names: vec![
                    ImportName {
                        name: "path".to_string(),
                        alias: None,
                    },
                    ImportName {
                        name: "getcwd".to_string(),
                        alias: Some("cwd".to_string()),
                    },
                    ImportName {
                        name: "sep".to_string(),
                        alias: None,
                    },
                ],
            })])
        );
    }

    #[test]
    fn test_parse_from_import_dotted_module() {
        let program = parse_source("from pkg.sub.mod import x as y;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::FromImport {
                module: vec![
                    "pkg".to_string(),
                    "sub".to_string(),
                    "mod".to_string(),
                ],
                names: vec![ImportName {
                    name: "x".to_string(),
                    alias: Some("y".to_string()),
                }],
            })])
        );
    }

    #[test]
    fn test_parse_from_import_star() {
        let program = parse_source("from math import *;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::FromImport {
                module: vec!["math".to_string()],
                names: vec![ImportName {
                    name: "*".to_string(),
                    alias: None,
                }],
            })])
        );
    }

    #[test]
    fn test_parse_from_import_missing_module_errors() {
        // `from 123 import x` — 123 is a literal, not an identifier.
        let mut lexer = Lexer::new("from 123 import x;");
        let tokens = lexer.lex().expect("lex failed");
        let mut p = Parser::new(tokens);
        assert!(p.parse().is_err());
    }
}
