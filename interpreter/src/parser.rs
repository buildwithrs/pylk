use crate::{
    ast::{
        AssignOperator, AssignTarget, BinaryOp, Block, ClassMember, CompareOp, DictEntry, Expr,
        ImportName, LiteralExpr, LogicalOp, Node, Parameter, Program, Stmt, UnaryOp,
    },
    errors::ParserError,
    lexer::{self, Keyword, Token, TokenType},
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

    fn is_assign_op(t: &Token) -> bool {
        matches!(
            t,
            Token::Assign
                | Token::PlusAssign
                | Token::MinusAssign
                | Token::MulAssign
                | Token::DivAssign
                | Token::FloorAssign
                | Token::ModAssign
                | Token::PowAssign
                | Token::MatMulAssign
                | Token::BitAndAssign
                | Token::BitOrAssign
                | Token::BitXorAssign
                | Token::ShlAssign
                | Token::ShrAssign
        )
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
        let cur = self.peek().ok_or_else(|| ParserError::EOF)?;

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
            Token::LBrace => Ok(Stmt::Block(self.parse_block()?)),
            _ => self.parse_assignment_stmt(),
        }
    }

    /// Parses `import a.b.c [as alias]`. The leading `import` token
    /// has already been consumed by `parse_stmt`.
    fn parse_import(&mut self) -> Result<Stmt, ParserError> {
        self.advance(); // consume 'import'

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
        self.advance();

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

    fn expect_token(&mut self, expect_t: Token) -> Result<Token, ParserError> {
        match self.advance() {
            Some(t) => {
                if t != expect_t {
                    Ok(t)
                } else {
                    Err(ParserError::ExpectToken(expect_t, t))
                }
            }
            None => Err(ParserError::EOF),
        }
    }

    fn expect_assigns(&mut self) -> Result<Token, ParserError> {
        match self.advance() {
            Some(t) => {
                if lexer::assign_tokens().contains(&t) {
                    return Ok(t);
                }

                return Err(ParserError::UnsupportToken(t));
            }
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
        self.advance();

        let class_name = self.expect_ident()?;
        let _ = self.consume(TokenType::LBrace)?;

        let mut members = Vec::new();
        while !self.check(&Token::RBrace) {
            let next = self.peek();

            let member = match next {
                Some(nt) => match nt {
                    Token::Kw(Keyword::Def) => Ok(ClassMember::FuncDecl(self.parse_function()?)),
                    _ => {
                        if let Token::Ident(name) = nt {
                            let next = self.peek_next().ok_or_else(|| ParserError::EOF)?;
                            if Self::is_assign_op(&next) {
                                Ok(ClassMember::Assign(self.parse_simple_assign()?))
                            } else {
                                Ok(ClassMember::ExprStmt(self.parse_expression()?))
                            }
                        } else {
                            Ok(ClassMember::ExprStmt(self.parse_expression()?))
                        }
                    }
                },

                None => Err(ParserError::EOF),
            };

            members.push(member?);
        }

        Ok(Stmt::Class {
            name: class_name,
            members: members,
        })
    }

    fn parse_function(&mut self) -> Result<Stmt, ParserError> {
        self.advance();

        let fn_name = self.expect_ident()?;
        let _ = self.consume(TokenType::LParen)?;

        // get arguments
        let mut args = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let name = self.expect_ident()?;
                args.push(Parameter {
                    name,
                    default: None,
                });
                if !self.check(&Token::Comma) {
                    break;
                }

                self.advance();
                if !self.check(&Token::RParen) {
                    break;
                }
            }
        }

        self.consume(TokenType::RParen)?;

        let body = self.parse_block()?;
        Ok(Stmt::Func {
            name: fn_name,
            param_list: args,
            body: body,
        })
    }

    fn parse_block(&mut self) -> Result<Block, ParserError> {
        self.consume(TokenType::LBrace)?;

        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.advance();

        Ok(Block(stmts))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParserError> {
        self.advance();

        let cond = self.parse_expression()?;
        let then = self.parse_block()?;
        let mut elifs = Vec::new();
        while self.check(&Token::Kw(Keyword::Elif)) {
            self.advance();

            let expr = self.parse_expression()?;
            let blk = self.parse_block()?;
            elifs.push((Box::new(expr), blk));
        }

        let mut else_block: Option<Block> = None;
        if self.check(&Token::Kw(Keyword::Else)) {
            self.advance();
            else_block = Some(self.parse_block()?);
        }

        Ok(Stmt::If {
            cond: Box::new(cond),
            then,
            elif_branches: elifs,
            else_branch: else_block,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParserError> {
        let _ = self.advance();

        let expr = self.parse_expression()?;
        Ok(Stmt::While {
            cond: Box::new(expr),
            body: self.parse_block()?,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParserError> {
        self.advance();

        let ident = self.expect_ident()?;

        if !self.check(&Token::Kw(Keyword::In)) {
            let cur = self.peek().unwrap();
            return Err(ParserError::ExpectToken(Token::Kw(Keyword::In), cur));
        }
        self.advance();

        let expr = self.parse_expression()?;
        let block = self.parse_block()?;
        Ok(Stmt::For {
            loop_var: ident,
            iter_expr: Box::new(expr),
            body: block,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParserError> {
        self.advance();

        let ret: Option<Expr> = if self.check(&Token::Semi) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Stmt::Return { value: ret })
    }

    fn parse_assignment_stmt(&mut self) -> Result<Stmt, ParserError> {
        println!("parsing assignment...");

        let assign_target = self.parse_expression()?;
        println!("parsing assignment, assign_target: {}", assign_target);

        if !self.is_end() {
            let next = self.peek().ok_or_else(|| ParserError::EOF)?;
            if Self::is_assign_op(&next) {
                self.advance();

                let op = AssignOperator::from(next);
                match assign_target {
                    Expr::Attribute { target, field_name } => {
                        return Ok(Stmt::Assign {
                            target: AssignTarget::Attribute {
                                object: target,
                                field: field_name,
                            },
                            op,
                            expr: Box::new(self.parse_expression()?),
                        });
                    }
                    Expr::Ident(x) => {
                        return Ok(Stmt::Assign {
                            target: AssignTarget::Name(x),
                            op,
                            expr: Box::new(self.parse_expression()?),
                        });
                    }

                    Expr::Slice { name, start, .. } => {
                        return Ok(Stmt::Assign {
                            target: AssignTarget::Index {
                                object: name,
                                index: start.unwrap(),
                            },
                            op,
                            expr: Box::new(self.parse_expression()?),
                        });
                    }

                    Expr::TupleLiteral(args) => {
                        if args.len() == 0 {
                            return Err(ParserError::InvalidAssignTarget1(
                                "no tuple args".to_string(),
                            ));
                        }

                        let mut tuple_args = vec![];
                        for arg in args {
                            match arg {
                                Expr::Ident(x) => {
                                    tuple_args.push(x);
                                }
                                _ => {
                                    return Err(ParserError::InvalidAssignTarget1(
                                        "tuple arg is not ident".to_string(),
                                    ));
                                }
                            }
                        }

                        return Ok(Stmt::Assign {
                            target: AssignTarget::Tuple(tuple_args),
                            op,
                            expr: Box::new(self.parse_expression()?),
                        });
                    }
                    _ => {
                        return Err(ParserError::InvalidAssignTarget1(format!(
                            "unsupported assign target: {}",
                            assign_target
                        )));
                    }
                }
            } else {
                return Ok(Stmt::ExprStmt(Box::new(assign_target)));
            }
        }

        Ok(Stmt::ExprStmt(Box::new(assign_target)))
    }

    // x = expression
    fn parse_simple_assign(&mut self) -> Result<Stmt, ParserError> {
        let ident = self.expect_ident()?;
        let op = AssignOperator::from(self.expect_assigns()?);
        let value = self.parse_expression()?;

        self.advance();
        Ok(Stmt::Assign {
            target: AssignTarget::Name(ident),
            op,
            expr: Box::new(value),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParserError> {
        let cur = self.peek().ok_or_else(|| ParserError::EOF)?;
        match cur {
            Token::Kw(Keyword::Lambda) => self.parse_lambda(),
            _ => self.parse_ternary_expr(),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParserError> {
        self.advance();

        let mut parameters = vec![self.expect_ident()?];
        while self.check(&Token::Comma) {
            self.advance();

            parameters.push(self.expect_ident()?);
        }

        self.consume(TokenType::Colon)?;

        let expr = self.parse_expression()?;
        Ok(Expr::Lambda {
            param_list: parameters,
            expression: Box::new(expr),
        })
    }

    fn parse_ternary_expr(&mut self) -> Result<Expr, ParserError> {
        println!("parsing ternary...");

        let logical_or = self.parse_logical_or()?;

        println!("logical_or: {}", logical_or);

        if self.check(&Token::Kw(Keyword::If)) {
            self.advance();

            let expr = self.parse_logical_or()?;
            self.expect_token(Token::Kw(Keyword::Else))?;

            let else_branch = self.parse_expression()?;
            return Ok(Expr::Ternary {
                true_expr: Box::new(logical_or),
                test: Box::new(expr),
                else_expr: Box::new(else_branch),
            });
        }

        Ok(logical_or)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_logical_and()?;
        if self.check(&Token::Kw(Keyword::Or)) {
            self.advance();

            let right = self.parse_logical_and()?;
            return Ok(Expr::Logical {
                op: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_equality()?;
        if self.check(&Token::Kw(Keyword::And)) {
            self.advance();

            let right = self.parse_equality()?;
            return Ok(Expr::Logical {
                op: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParserError> {
        let rel = self.parse_relational()?;
        if self.check(&Token::Eq) || self.check(&Token::Ne) {
            self.advance();

            let cur = self.peek().ok_or(ParserError::EOF)?;
            let op = if Token::Eq == cur {
                CompareOp::Eq
            } else {
                CompareOp::NotEq
            };

            return Ok(Expr::Compare {
                op,
                left: Box::new(rel),
                right: Box::new(self.parse_relational()?),
            });
        }

        Ok(rel)
    }

    fn parse_relational(&mut self) -> Result<Expr, ParserError> {
        let bit_or = self.parse_bitwise_or()?;

        if self.match_tokens(&[Token::Le, Token::Lt, Token::Ge, Token::Gt]) {
            let cur = self.advance().ok_or(ParserError::EOF)?;
            let op = match cur {
                Token::Lt => CompareOp::Lt,
                Token::Le => CompareOp::Le,
                Token::Ge => CompareOp::Ge,
                Token::Gt => CompareOp::Gt,
                _ => CompareOp::Gt,
            };

            return Ok(Expr::Compare {
                op,
                left: Box::new(bit_or),
                right: Box::new(self.parse_bitwise_or()?),
            });
        }

        Ok(bit_or)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParserError> {
        let xor = self.parse_bitwise_xor()?;

        if self.check(&Token::BitOr) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            return Ok(Expr::BinaryExpr {
                left: Box::new(xor),
                op: BinaryOp::BitOr,
                right: Box::new(right),
            });
        }

        Ok(xor)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_bitwise_and()?;

        if self.check(&Token::BitXor) {
            self.advance();
            let right = self.parse_bitwise_and()?;

            return Ok(Expr::BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitXor,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_shift()?;

        if self.check(&Token::BitAnd) {
            self.advance();
            let right = self.parse_shift()?;

            return Ok(Expr::BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    /// << and >>
    fn parse_shift(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_additive()?;

        if self.match_tokens(&[Token::Shl, Token::Shr]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_additive()?;

            let op = if cur == Token::Shl {
                BinaryOp::Shl
            } else {
                BinaryOp::Shr
            };

            return Ok(Expr::BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    /// +, -
    fn parse_additive(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_multiplicative()?;

        if self.match_tokens(&[Token::Plus, Token::Minus]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_multiplicative()?;

            let op = if cur == Token::Plus {
                BinaryOp::Plus
            } else {
                BinaryOp::Minus
            };

            return Ok(Expr::BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParserError> {
        let left = self.parse_unary()?;

        if self.match_tokens(&[Token::Mul, Token::Div, Token::FloorDiv, Token::Mod]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_unary()?;

            let op = match cur {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                Token::FloorDiv => BinaryOp::FloorDiv,
                _ => BinaryOp::Mod,
            };

            return Ok(Expr::BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_tokens(&[Token::Plus, Token::Minus, Token::BitNot]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_unary()?;
            let op = match cur {
                Token::Plus => UnaryOp::Plus,
                Token::Minus => UnaryOp::Minus,
                _ => UnaryOp::BitNot,
            };

            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
            });
        }

        self.parse_power()
    }

    /// **
    fn parse_power(&mut self) -> Result<Expr, ParserError> {
        let postfix = self.parse_postfix()?;
        if self.check(&Token::Pow) {
            self.advance();

            return Ok(Expr::Power {
                left: Box::new(postfix),
                right: Box::new(self.parse_unary()?),
            });
        }
        Ok(postfix)
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParserError> {
        let pri = self.parse_primary()?;

        if self.check(&Token::LParen) {
            self.advance();

            let mut args = Vec::new();
            loop {
                if self.check(&Token::RParen) {
                    break;
                }

                args.push(self.parse_expression()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.consume(TokenType::RParen)?;

            return Ok(Expr::FuncCall {
                fn_expr: Box::new(pri),
                args,
            });
        } else if self.check(&Token::LBracket) {
            self.advance();

            let (start, end, step) = self.parse_slice()?;
            return Ok(Expr::Slice {
                name: Box::new(pri),
                start,
                end,
                step,
            });
        } else if self.check(&Token::Dot) {
            self.advance();
            let attr = self.expect_ident()?;
            return Ok(Expr::Attribute {
                target: Box::new(pri),
                field_name: attr,
            });
        }

        Ok(pri)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        let t = self.advance().ok_or_else(|| ParserError::EOF)?;

        match self.parse_literal(t.clone()) {
            Ok(v) => Ok(v),
            Err(_) => match t {
                Token::Ident(id) => Ok(Expr::Ident(id)),
                Token::LBracket => self.parse_list_literal(),
                Token::LBrace => self.parse_dict_literal(),
                Token::LParen => self.parse_tuple_or_grouped(),
                _ => Err(ParserError::UnsupportToken(t)),
            },
        }
    }

    fn parse_slice(
        &mut self,
    ) -> Result<(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>), ParserError> {
        let start = self.parse_expression()?;
        self.consume(TokenType::RBracket)?;

        Ok((Some(Box::new(start)), None, None))
    }

    /// ()
    /// (expr, expr)
    /// (expr)
    /// `()` | `( expr )` | `( expr "," )` | `( expr "," expr { "," expr } )`
    ///
    /// Disambiguates grouped expressions from tuples using the
    /// trailing-comma rule: `(x)` is just `x`, but `(x,)` is a
    /// one-element tuple. `()` is the empty tuple.
    fn parse_tuple_or_grouped(&mut self) -> Result<Expr, ParserError> {
        // empty tuple: `()`
        if self.check(&Token::RParen) {
            self.advance();
            return Ok(Expr::TupleLiteral(Vec::new()));
        }

        let first = self.parse_expression()?;

        // grouped expression: `(expr)` -- no comma after the inner expr
        if !self.check(&Token::Comma) {
            self.consume(TokenType::RParen)?;
            return Ok(first);
        }

        // tuple: at least one element already in `first`
        let mut elements = vec![first];
        while self.check(&Token::Comma) {
            self.advance();
            // allow trailing comma: `(a, b,)`
            if self.check(&Token::RParen) {
                break;
            }
            elements.push(self.parse_expression()?);
        }
        self.consume(TokenType::RParen)?;
        Ok(Expr::TupleLiteral(elements))
    }

    /// `[ expr { "," expr } ]`
    fn parse_list_literal(&mut self) -> Result<Expr, ParserError> {
        let mut elements = Vec::new();
        if !self.check(&Token::RBracket) {
            elements.push(self.parse_expression()?);
            while self.check(&Token::Comma) {
                self.advance();

                // allow trailing comma: `[a, b,]`
                if self.check(&Token::RBracket) {
                    break;
                }
                elements.push(self.parse_expression()?);
            }
        }

        self.consume(TokenType::RBracket)?;
        Ok(Expr::ListLiteral(elements))
    }

    /// `{ expr ":" expr { "," expr ":" expr } }`
    fn parse_dict_literal(&mut self) -> Result<Expr, ParserError> {
        let mut entries = Vec::new();
        if !self.check(&Token::RBrace) {
            let key = self.parse_expression()?;
            self.consume(TokenType::Colon)?;
            let value = self.parse_expression()?;

            entries.push(DictEntry {
                key: Box::new(key),
                value: Box::new(value),
            });

            while self.check(&Token::Comma) {
                self.advance();
                // allow trailing comma: `{a: 1,}`
                if self.check(&Token::RBrace) {
                    break;
                }

                let key = self.parse_expression()?;
                self.consume(TokenType::Colon)?;
                let value = self.parse_expression()?;

                entries.push(DictEntry {
                    key: Box::new(key),
                    value: Box::new(value),
                });
            }
        }
        self.consume(TokenType::RBrace)?;
        Ok(Expr::DictLiteral(entries))
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
        let t = self.advance().ok_or_else(|| ParserError::EOF)?;
        if t.token_type() != token_type {
            return Err(ParserError::ExpectTokenType(token_type, t.token_type()));
        }

        Ok(t)
    }

    fn consume_oneof_types(&mut self, token_types: &[TokenType]) -> Result<Token, ParserError> {
        let tt = self.advance().ok_or_else(|| ParserError::EOF)?;
        if !token_types.contains(&tt.token_type()) {
            return Err(ParserError::ExpectTokenType(
                token_types[0],
                tt.token_type(),
            ));
        }

        Ok(tt)
    }

    fn match_tokens(&self, tokens: &[Token]) -> bool {
        match self.peek() {
            Some(tk) => tokens.contains(&tk),
            None => false,
        }
    }

    fn peek(&self) -> Option<Token> {
        if self.is_end() {
            None
        } else {
            Some(self.tokens[self.current].clone())
        }
    }

    fn peek_next(&self) -> Option<Token> {
        if self.current + 1 >= self.tokens.len() {
            None
        } else {
            Some(self.tokens[self.current + 1].clone())
        }
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
        ast::{
            AssignOperator, AssignTarget, Block, Expr, ImportName, LiteralExpr, Node, Program, Stmt,
        },
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

        // let nodes = vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Assign {
        //     target: AssignTarget::Name("x".to_string()),
        //     op: AssignOperator::Assign,
        //     expr: Box::new(Expr::Literal(LiteralExpr::Float(123.0))),
        // })))];
        // assert_eq!(program, Program(nodes));
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
                module: vec!["pkg".to_string(), "sub".to_string(), "mod".to_string(),],
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

    #[test]
    fn test_parse_function() {
        let mut lexer = Lexer::new(
            r#"def test() {
            print("Hello");
        }"#,
        );
        let tokens = lexer.lex().expect("lex failed");

        println!("tokens: {:?}", tokens);

        let mut p = Parser::new(tokens);

        let program = p.parse().expect("parse failed");

        let nodes: Vec<Node> = vec![Node::new(Stmt::Func {
            name: "test".to_string(),
            param_list: vec![],
            body: Block(vec![Stmt::ExprStmt(Box::new(Expr::FuncCall {
                fn_expr: Box::new(Expr::Ident("print".to_string())),
                args: vec![Expr::Literal(LiteralExpr::Str("Hello".to_string()))],
            }))]),
        })];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_call() {
        let mut lexer = Lexer::new(r#"nice(123);"#);
        let tokens = lexer.lex().expect("lex failed");

        println!("tokens: {:?}", tokens);

        let mut p = Parser::new(tokens);
        let program = p.parse().expect("parse failed");
        let nodes: Vec<Node> = vec![Node::new(Stmt::ExprStmt(Box::new(Expr::FuncCall {
            fn_expr: Box::new(Expr::Ident("nice".to_string())),
            args: vec![Expr::Literal(LiteralExpr::Float(123.0))],
        })))];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_attribute() {
        let mut lexer = Lexer::new(r#"image.width;"#);
        let tokens = lexer.lex().expect("lex failed");

        println!("tokens: {:?}", tokens);

        let mut p = Parser::new(tokens);
        let program = p.parse().expect("parse failed");
        let nodes: Vec<Node> = vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Attribute {
            target: Box::new(Expr::Ident("image".to_string())),
            field_name: "width".to_string(),
        })))];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_assign1() {
        let mut lexer = Lexer::new(r#"x=123;"#);
        let tokens = lexer.lex().expect("lex failed");
        println!("tokens: {:?}", tokens);

        let mut p = Parser::new(tokens);
        let program = p.parse().expect("parse failed");
        let nodes: Vec<Node> = vec![Node::new(Stmt::Assign {
            target: AssignTarget::Name("x".to_string()),
            op: AssignOperator::Assign,
            expr: Box::new(Expr::Literal(LiteralExpr::Float(123.0))),
        })];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_assign2() {
        let mut lexer = Lexer::new(r#"x.y=123;"#);
        let tokens = lexer.lex().expect("lex failed");
        println!("tokens: {:?}", tokens);

        let mut p = Parser::new(tokens);
        let program = p.parse().expect("parse failed");
        let nodes: Vec<Node> = vec![Node::new(Stmt::Assign {
            target: AssignTarget::Attribute {
                object: Box::new(Expr::Ident("x".to_string())),
                field: "y".to_string(),
            },
            op: AssignOperator::Assign,
            expr: Box::new(Expr::Literal(LiteralExpr::Float(123.0))),
        })];
        assert_eq!(program, Program(nodes));
    }

    #[test]
    fn test_parse_assign3() {
        let mut lexer = Lexer::new(r#"arr[0]='X';"#);
        let tokens = lexer.lex().expect("lex failed");
        println!("tokens: {:?}", tokens);

        let mut p = Parser::new(tokens);
        let program = p.parse().expect("parse failed");
        let nodes: Vec<Node> = vec![Node::new(Stmt::Assign {
            target: AssignTarget::Index {
                object: Box::new(Expr::Ident("arr".to_string())),
                index: Box::new(Expr::Literal(LiteralExpr::Int(0))),
            },
            op: AssignOperator::Assign,
            expr: Box::new(Expr::Literal(LiteralExpr::Str("X".to_string()))),
        })];
        assert_eq!(program, Program(nodes));
    }
}
