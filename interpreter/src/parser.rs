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
            Token::LBrace => {
                // `{` can start a block or a dict literal expression
                // statement. Per the grammar note, we disambiguate by
                // trying the dict path first; if it doesn't parse as a
                // dict entry, fall back to block.
                let saved = self.current;
                match self.try_parse_dict_literal_at_stmt() {
                    Ok(expr) => Ok(Stmt::ExprStmt(Box::new(expr))),
                    Err(_) => {
                        self.current = saved;
                        Ok(Stmt::Block(self.parse_block()?))
                    }
                }
            }
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
                if t == expect_t {
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
                    Token::Kw(Keyword::Def) => {
                        println!("parse class function");
                        Ok(ClassMember::FuncDecl(self.parse_function()?))
                    }
                    _ => {
                        if let Token::Ident(name) = nt {
                            println!("parse class simple assign: {name}");

                            let next = self.peek_next().ok_or_else(|| ParserError::EOF)?;
                            if Self::is_assign_op(&next) {
                                self.advance(); // skip name
                                self.advance(); // skip =
                                Ok(ClassMember::Assign(self.parse_simple_assign(name, next)?))
                            } else {
                                println!("parse class other expr: {name}");
                                Ok(ClassMember::ExprStmt(self.parse_expression()?))
                            }
                        } else {
                            println!("parse class other expr: {nt}");
                            Ok(ClassMember::ExprStmt(self.parse_expression()?))
                        }
                    }
                },

                None => Err(ParserError::EOF),
            };

            members.push(member?);
        }

        self.consume(TokenType::RBrace)?;

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

                if self.check(&Token::Comma) {
                    self.advance();
                }

                if self.check(&Token::RParen) {
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
        let assign_target = self.parse_expression()?;

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

                    Expr::Index { target, index } => {
                        return Ok(Stmt::Assign {
                            target: AssignTarget::Index {
                                object: target,
                                index,
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
    fn parse_simple_assign(&mut self, name: String, op: Token) -> Result<Stmt, ParserError> {
        let op = AssignOperator::from(op);
        let value = self.parse_expression()?;
        if self.check(&Token::Semi) {
            self.advance();
        }

        Ok(Stmt::Assign {
            target: AssignTarget::Name(name),
            op,
            expr: Box::new(value),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParserError> {
        println!("parse expression...");

        let cur = self.peek().ok_or_else(|| ParserError::EOF)?;
        match cur {
            Token::Kw(Keyword::Lambda) => self.parse_lambda(),
            _ => self.parse_ternary_expr(),
        }
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParserError> {
        println!("parse lambda...");

        self.advance();

        let mut parameters = vec![self.expect_ident()?];
        while self.check(&Token::Comma) {
            self.advance();

            parameters.push(self.expect_ident()?);
        }

        self.consume(TokenType::Colon)?;

        let expr = self.parse_expression()?;
        println!("parse lambda...: expression: {expr}");
        Ok(Expr::Lambda {
            param_list: parameters,
            expression: Box::new(expr),
        })
    }

    fn parse_ternary_expr(&mut self) -> Result<Expr, ParserError> {
        println!("parse ternary expr...");

        let logical_or = self.parse_logical_or()?;

        println!("parse ternary expr...: logical_or: {logical_or}");

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

    /// `or`   (left-associative)
    fn parse_logical_or(&mut self) -> Result<Expr, ParserError> {
        println!("parse logical_or...");

        let mut left = self.parse_logical_and()?;
        while self.check(&Token::Kw(Keyword::Or)) {
            self.advance();

            let right = self.parse_logical_and()?;
            left = Expr::Logical {
                op: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// `and`   (left-associative)
    fn parse_logical_and(&mut self) -> Result<Expr, ParserError> {
        println!("parse logical_and...");

        let mut left = self.parse_not_expr()?;
        println!("parse logical_and...: {left}");

        while self.check(&Token::Kw(Keyword::And)) {
            self.advance();

            let right = self.parse_not_expr()?;
            left = Expr::Logical {
                op: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<Expr, ParserError> {
        println!("parse_not_expr...");

        if self.check(&Token::Kw(Keyword::Not)) {
            self.advance();
            Ok(Expr::LogicalNot(Box::new(self.parse_not_expr()?)))
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParserError> {
        println!("parse comparison...");

        let bit_or = self.parse_bitwise_or()?;
        println!("parse comparison...: {bit_or}");

        if self.match_tokens(&[
            Token::Eq,
            Token::Ne,
            Token::Le,
            Token::Lt,
            Token::Ge,
            Token::Gt,
        ]) {
            let cur = self.advance().ok_or(ParserError::EOF)?;
            let op = match cur {
                Token::Eq => CompareOp::Eq,
                Token::Ne => CompareOp::NotEq,
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

    /// |   (left-associative)
    fn parse_bitwise_or(&mut self) -> Result<Expr, ParserError> {
        println!("parse bitwise_or...");

        let mut left = self.parse_bitwise_xor()?;
        println!("parse bitwise_or...: left: {left}");

        while self.check(&Token::BitOr) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expr::BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitOr,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// ^   (left-associative)
    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParserError> {
        println!("parse bitwise_xor...");

        let mut left = self.parse_bitwise_and()?;
        println!("parse bitwise_xor...: left: {left}");

        while self.check(&Token::BitXor) {
            self.advance();
            let right = self.parse_bitwise_and()?;

            left = Expr::BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitXor,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// &   (left-associative)
    fn parse_bitwise_and(&mut self) -> Result<Expr, ParserError> {
        println!("parse bitwise_and...");

        let mut left = self.parse_shift()?;
        println!("parse bitwise_and...: left: {left}");

        while self.check(&Token::BitAnd) {
            self.advance();
            let right = self.parse_shift()?;

            left = Expr::BinaryExpr {
                left: Box::new(left),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// << and >>   (left-associative)
    fn parse_shift(&mut self) -> Result<Expr, ParserError> {
        println!("parse shift...");

        let mut left = self.parse_additive()?;
        println!("parse shift...: left: {left}");

        while self.match_tokens(&[Token::Shl, Token::Shr]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_additive()?;

            let op = if cur == Token::Shl {
                BinaryOp::Shl
            } else {
                BinaryOp::Shr
            };

            left = Expr::BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// +, -   (left-associative)
    fn parse_additive(&mut self) -> Result<Expr, ParserError> {
        println!("parse additive...");

        let mut left = self.parse_multiplicative()?;
        println!("parse additive...: left: {left}");

        while self.match_tokens(&[Token::Plus, Token::Minus]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_multiplicative()?;

            let op = if cur == Token::Plus {
                BinaryOp::Plus
            } else {
                BinaryOp::Minus
            };

            left = Expr::BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// *, /, //, %   (left-associative)
    fn parse_multiplicative(&mut self) -> Result<Expr, ParserError> {
        println!("parse multiplicative...");

        let mut left = self.parse_unary()?;
        println!("parse multiplicative...: left: {left}");

        while self.match_tokens(&[Token::Mul, Token::Div, Token::FloorDiv, Token::Mod]) {
            let cur = self.advance().ok_or_else(|| ParserError::EOF)?;
            let right = self.parse_unary()?;

            let op = match cur {
                Token::Mul => BinaryOp::Mul,
                Token::Div => BinaryOp::Div,
                Token::FloorDiv => BinaryOp::FloorDiv,
                _ => BinaryOp::Mod,
            };

            left = Expr::BinaryExpr {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParserError> {
        println!("parse unary...");

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

        let power = self.parse_power()?;
        println!("parse unary...: power: {power}");
        Ok(power)
    }

    /// **
    fn parse_power(&mut self) -> Result<Expr, ParserError> {
        println!("parse power...");

        let postfix = self.parse_postfix()?;
        println!("parse power...: postfix: {postfix}");

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
        println!("parse postfix...");

        let pri = self.parse_primary()?;
        println!("parse postfix...: primary: {pri}");
        let result = self.parse_postfix_op(pri)?;
        println!("parse postfix...: result: {result}");
        Ok(result)
    }

    fn parse_postfix_op(&mut self, pri: Expr) -> Result<Expr, ParserError> {
        match self.peek() {
            Some(Token::LParen) => {
                let fn_call = self.parse_func_call(pri)?;
                Ok(self.parse_postfix_op(fn_call)?)
            }
            Some(Token::LBracket) => {
                self.advance();
                let slice_expr = self.parse_slice(pri)?;
                if self.check(&Token::Semi) {
                    self.advance();
                }

                Ok(self.parse_postfix_op(slice_expr)?)
            }
            Some(Token::Dot) => {
                self.advance();
                let attr_name = self.expect_ident()?;
                if self.check(&Token::Semi) {
                    self.advance();
                }

                let attr = Expr::Attribute {
                    target: Box::new(pri),
                    field_name: attr_name,
                };
                Ok(self.parse_postfix_op(attr)?)
            },
            // No more postfix ops to apply — return the accumulated
            // expression. The caller (parse_postfix → parse_power →
            // parse_unary → ...) will handle the next token.
            _ => Ok(pri),
        }
    }

    fn parse_func_call(&mut self, pri: Expr) -> Result<Expr, ParserError> {
        println!("parsing func calll...: {pri}");

        self.advance();

        let mut args = Vec::new();
        loop {
            let t = self.peek();
            println!("[parse_func_call] checking {:?}", t);

            if self.check(&Token::RParen) {
                break;
            }

            println!("parse arg expr...");
            let arg = self.parse_expression()?;
            println!("pushing arg: {arg}");

            args.push(arg);

            println!("consume comma...");
            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        self.consume(TokenType::RParen)?;
        if self.check(&Token::Semi) {
            self.advance();
        }

        Ok(Expr::FuncCall {
            fn_expr: Box::new(pri),
            args,
        })
    }

    fn parse_primary(&mut self) -> Result<Expr, ParserError> {
        println!("parse primary...");

        let t = self.advance().ok_or_else(|| ParserError::EOF)?;
        println!("parse primary...: token: {t}");

        let result = match self.parse_literal(t.clone()) {
            Ok(v) => Ok(v),
            Err(_) => match t {
                Token::Ident(id) => Ok(Expr::Ident(id)),
                Token::LBracket => self.parse_list_literal(),
                Token::LBrace => self.parse_dict_literal(),
                Token::LParen => self.parse_tuple_or_grouped(),
                _ => Err(ParserError::UnsupportToken(t)),
            },
        }?;
        println!("parse primary...: result: {result}");
        Ok(result)
    }

    /// Parses `pri[<expr>]`, `pri[<start>:]`, `pri[:<end>]`,
    /// `pri[<start>:<end>]`, `pri[:<end>:<step>]`, and
    /// `pri[<start>:<end>:<step>]`. Returns `Expr::Index` for the
    /// index-access form (no `:`), otherwise `Expr::Slice`.
    ///
    /// The `start` bound can be absent (`[:`) or present (`[<expr>:`).
    /// From that point on the slice has the shape `[start:end:step?]`,
    /// so both forms share the same end/step tail and we only branch on
    /// whether the leading character is `:`.
    fn parse_slice(&mut self, pri: Expr) -> Result<Expr, ParserError> {
        println!("parse slice...");

        // Parse the optional start bound. A leading `:` means start is
        // absent; otherwise parse an expression and check whether it's
        // really an index access (no `:` follows).
        let start = if self.check(&Token::Colon) {
            self.advance();
            None
        } else {
            let expr = self.parse_expression()?;
            // `[expr]` with no `:` is an index, not a slice.
            if self.check(&Token::RBracket) {
                self.advance();
                let index = Expr::Index {
                    target: Box::new(pri),
                    index: Box::new(expr),
                };
                println!("parse slice...: index: {index}");
                return Ok(index);
            }
            self.consume(TokenType::Colon)?;
            Some(expr)
        };

        // Shared end/step tail for both `[:...]` and `[start:...]`.
        let end = if self.check(&Token::RBracket) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        // Optional `:step`. The two arms are intentionally asymmetric:
        // the step arm uses `consume(RBracket)` (so a missing `]`
        // becomes `ExpectTokenType`), while a bare `]` after the end
        // is matched via `peek()` (so a stray token there surfaces as
        // `InvalidSlice`). Existing tests pin this down.
        let step = match self.peek() {
            Some(Token::Colon) => {
                self.advance();
                let step = self.parse_expression()?;
                self.consume(TokenType::RBracket)?;
                Some(step)
            }
            Some(Token::RBracket) => {
                self.advance();
                None
            }
            Some(t) => return Err(ParserError::InvalidSlice(t)),
            None => return Err(ParserError::EOF),
        };

        let slice = Expr::Slice {
            name: Box::new(pri),
            start: start.map(Box::new),
            end: end.map(Box::new),
            step: step.map(Box::new),
        };
        println!("parse slice...: slice: {slice}");
        Ok(slice)
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
        println!("parse tuple_or_grouped...");

        // empty tuple: `()`
        if self.check(&Token::RParen) {
            self.advance();
            let empty = Expr::TupleLiteral(Vec::new());
            println!("parse tuple_or_grouped...: empty tuple: {empty}");
            return Ok(empty);
        }

        let first = self.parse_expression()?;

        // grouped expression: `(expr)` -- no comma after the inner expr
        if !self.check(&Token::Comma) {
            self.consume(TokenType::RParen)?;
            println!("parse tuple_or_grouped...: grouped: {first}");
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
        let tuple = Expr::TupleLiteral(elements);
        println!("parse tuple_or_grouped...: tuple: {tuple}");
        Ok(tuple)
    }

    /// `[ expr { "," expr } ]`
    fn parse_list_literal(&mut self) -> Result<Expr, ParserError> {
        println!("parse list_literal...");

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
        let list = Expr::ListLiteral(elements);
        println!("parse list_literal...: list: {list}");
        Ok(list)
    }

    /// `{ expr ":" expr { "," expr ":" expr } }`
    fn parse_dict_literal(&mut self) -> Result<Expr, ParserError> {
        println!("parse dict_literal...");

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
        let dict = Expr::DictLiteral(entries);
        println!("parse dict_literal...: dict: {dict}");
        Ok(dict)
    }

    /// Attempts to parse a `{ ... }` at statement position as a dict
    /// literal expression statement. Returns `Err` if the contents are
    /// not shaped like a dict entry, signalling the caller to fall
    /// back to block parsing.
    ///
    /// Per the grammar, an empty `{}` is an empty dict literal (the
    /// grammar notes "empty set has no literal form"). Otherwise we
    /// require the first inner expression to be followed by `:`, so
    /// blocks like `{x; y;}` are left untouched.
    fn try_parse_dict_literal_at_stmt(&mut self) -> Result<Expr, ParserError> {
        println!("try_parse_dict_literal_at_stmt...");

        self.consume(TokenType::LBrace)?;

        // `{}` → empty dict.
        if self.check(&Token::RBrace) {
            self.advance();
            let empty = Expr::DictLiteral(Vec::new());
            println!("try_parse_dict_literal_at_stmt...: empty dict: {empty}");
            return Ok(empty);
        }

        // Parse the first key expression. If the next token isn't `:`,
        // this is a block, not a dict.
        let key = self.parse_expression()?;
        if !self.check(&Token::Colon) {
            return Err(ParserError::UnsupportToken(
                self.peek().unwrap_or(Token::Eof),
            ));
        }
        self.advance();

        let value = self.parse_expression()?;
        let mut entries = vec![DictEntry {
            key: Box::new(key),
            value: Box::new(value),
        }];

        while self.check(&Token::Comma) {
            self.advance();
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
        self.consume(TokenType::RBrace)?;
        let dict = Expr::DictLiteral(entries);
        println!("try_parse_dict_literal_at_stmt...: dict: {dict}");
        Ok(dict)
    }

    fn parse_literal(&mut self, t: Token) -> Result<Expr, ParserError> {
        println!("parse literal...: token: {t}");

        let result = match t {
            Token::Str(s) => Ok(Expr::Literal(LiteralExpr::Str(s))),
            Token::Int(i) => Ok(Expr::Literal(LiteralExpr::Int(i))),
            Token::Float(f) => Ok(Expr::Literal(LiteralExpr::Float(f))),
            Token::Bool(b) => Ok(Expr::Literal(LiteralExpr::Boolean(b))),
            Token::Kw(Keyword::None) => Ok(Expr::Literal(LiteralExpr::None)),
            _ => Err(ParserError::UnsupportToken(t)),
        }?;
        println!("parse literal...: result: {result}");
        Ok(result)
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
            AssignOperator, AssignTarget, BinaryOp, Block, ClassMember, CompareOp, Expr,
            ImportName, LiteralExpr, LogicalOp, Node, Program, Stmt, UnaryOp,
        },
        errors::ParserError,
        lexer::{Lexer, Token},
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
        //     expr: Box::new(Expr::Literal(LiteralExpr::Int(123))),
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
            args: vec![Expr::Literal(LiteralExpr::Int(123))],
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
            expr: Box::new(Expr::Literal(LiteralExpr::Int(123))),
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
            expr: Box::new(Expr::Literal(LiteralExpr::Int(123))),
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

    // -----------------------------------------------------------------
    // Tests for Parser::parse_slice.
    //
    // `parse_slice` is private, so each test drives it through the full
    // lex+parse pipeline on source code that, after the opening `[` is
    // consumed by `parse_postfix`, hands control to `parse_slice` with
    // the preceding expression as `pri`.
    //
    // Each shape maps to a distinct branch in `parse_slice`:
    //
    //   arr[1]       → Expr::Index                       (branch B1)
    //   arr[:]       → Slice{ --, --, -- }               (branch A1)
    //   arr[1:]      → Slice{ 1,  --, --}                (branch B2)
    //   arr[:5]      → Slice{ --, 5,  --}                (branch A2b)
    //   arr[1:5]     → Slice{ 1,  5,  --}                (branch B4)
    //   arr[:5:2]    → Slice{ --, 5,  2 }                (branch A2a)
    //   arr[1:5:2]   → Slice{ 1,  5,  2 }                (branch B3)
    // -----------------------------------------------------------------

    /// Variant of [`parse_source`] that surfaces the parser error so tests
    /// can assert on it.
    fn try_parse(source: &str) -> Result<Program, ParserError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().expect("lex failed");
        let mut p = Parser::new(tokens);
        p.parse()
    }

    #[test]
    fn test_parse_slice_index() {
        // arr[1] — single integer index, no slice. Exercises branch B1,
        // which terminates with `]` right after the start expression.
        let program = parse_source("arr[1];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Index {
                target: Box::new(Expr::Ident("arr".to_string())),
                index: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            })))])
        );
    }

    #[test]
    fn test_parse_slice_open_open() {
        // arr[:] — bare colon, no bounds at all. Exercises branch A1
        // (colon immediately followed by `]`), which produces the
        // most-degenerate slice with every bound None.
        let program = parse_source("arr[:];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Slice {
                name: Box::new(Expr::Ident("arr".to_string())),
                start: None,
                end: None,
                step: None,
            })))])
        );
    }

    #[test]
    fn test_parse_slice_start_only() {
        // arr[1:] — start bound present, end open. Exercises branch B2:
        // after parsing the start expression we see `:` then `]`.
        let program = parse_source("arr[1:];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Slice {
                name: Box::new(Expr::Ident("arr".to_string())),
                start: Some(Box::new(Expr::Literal(LiteralExpr::Int(1)))),
                end: None,
                step: None,
            })))])
        );
    }

    #[test]
    fn test_parse_slice_end_only() {
        // arr[:5] — start open, end present, no step. Exercises branch
        // A2b: leading `:`, parse end expression, then `]` (no second `:`).
        let program = parse_source("arr[:5];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Slice {
                name: Box::new(Expr::Ident("arr".to_string())),
                start: None,
                end: Some(Box::new(Expr::Literal(LiteralExpr::Int(5)))),
                step: None,
            })))])
        );
    }

    #[test]
    fn test_parse_slice_start_end() {
        // arr[1:5] — start and end present, no step. Exercises branch B4:
        // after start and `:` we parse end, then see `]` directly.
        let program = parse_source("arr[1:5];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Slice {
                name: Box::new(Expr::Ident("arr".to_string())),
                start: Some(Box::new(Expr::Literal(LiteralExpr::Int(1)))),
                end: Some(Box::new(Expr::Literal(LiteralExpr::Int(5)))),
                step: None,
            })))])
        );
    }

    #[test]
    fn test_parse_slice_end_step() {
        // arr[:5:2] — start open, end and step present. Exercises branch
        // A2a: leading `:`, parse end, see second `:`, parse step, `]`.
        let program = parse_source("arr[:5:2];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Slice {
                name: Box::new(Expr::Ident("arr".to_string())),
                start: None,
                end: Some(Box::new(Expr::Literal(LiteralExpr::Int(5)))),
                step: Some(Box::new(Expr::Literal(LiteralExpr::Int(2)))),
            })))])
        );
    }

    #[test]
    fn test_parse_slice_start_end_step() {
        // arr[1:5:2] — full slice with all three bounds. Exercises branch
        // B3: parse start, `:`, parse end, see second `:`, parse step, `]`.
        let program = parse_source("arr[1:5:2];");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(Expr::Slice {
                name: Box::new(Expr::Ident("arr".to_string())),
                start: Some(Box::new(Expr::Literal(LiteralExpr::Int(1)))),
                end: Some(Box::new(Expr::Literal(LiteralExpr::Int(5)))),
                step: Some(Box::new(Expr::Literal(LiteralExpr::Int(2)))),
            })))])
        );
    }

    #[test]
    fn test_parse_slice_invalid_after_end() {
        // arr[1:5,] — after the end expression we hit a `,` rather than
        // `:` or `]`. Both end-bound branches (A2 and B's tail) route the
        // unexpected token through `ParserError::InvalidSlice`.
        let err = try_parse("arr[1:5,];").expect_err("expected InvalidSlice error");
        match err {
            ParserError::InvalidSlice(Token::Comma) => {}
            other => panic!("expected InvalidSlice(Comma), got {:?}", other),
        }
    }

    #[test]
    fn test_parse_slice_invalid_after_step() {
        // arr[1:5:2,] — the step branch (B3) finishes with a hard
        // `consume(RBracket)` rather than a `peek()`-then-match, so a
        // trailing `,` surfaces as `ExpectTokenType(RBracket, Comma)`
        // instead of `InvalidSlice`. Recorded here so the asymmetry
        // between the step and non-step branches is pinned down.
        let err = try_parse("arr[1:5:2,];").expect_err("expected parser error");
        match err {
            ParserError::ExpectTokenType(_, _) => {}
            other => panic!("expected ExpectTokenType, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_slice_unterminated() {
        // `arr[` followed by nothing — once `[` is consumed in
        // `parse_postfix`, `parse_slice` tries to read an expression
        // for the start bound and falls off the end of the token
        // stream, surfacing `ParserError::EOF`.
        let err = try_parse("arr[").expect_err("expected EOF error");
        assert!(matches!(err, ParserError::EOF));
    }

    // -----------------------------------------------------------------
    // Expression tests.
    //
    // Each test lexes + parses a single `expr;` statement and unwraps
    // the inner `Expr` from the resulting `Stmt::ExprStmt`. This keeps
    // the assertions focused on the expression grammar and avoids
    // depending on any other statement-level machinery.
    //
    // Helper: pull the inner Expr out of a single-stmt program that
    // is expected to be `expr;`. Panics if the program has any other
    // shape — these helpers are only used on known-good inputs.
    // -----------------------------------------------------------------

    fn expr_of(source: &str) -> Expr {
        let program = parse_source(source);
        let mut nodes = program.0;
        assert_eq!(nodes.len(), 1, "expected exactly one node");
        match nodes.pop().unwrap().stmt {
            Stmt::ExprStmt(e) => *e,
            other => panic!("expected ExprStmt, got {:?}", other),
        }
    }

    // -- arithmetic --------------------------------------------------

    #[test]
    fn test_expr_add() {
        assert_eq!(
            expr_of("1 + 2;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                op: BinaryOp::Plus,
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_sub() {
        assert_eq!(
            expr_of("1 - 2;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                op: BinaryOp::Minus,
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_mul() {
        assert_eq!(
            expr_of("3 * 4;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(3))),
                op: BinaryOp::Mul,
                right: Box::new(Expr::Literal(LiteralExpr::Int(4))),
            }
        );
    }

    #[test]
    fn test_expr_div() {
        assert_eq!(
            expr_of("8 / 2;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(8))),
                op: BinaryOp::Div,
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_floor_div() {
        assert_eq!(
            expr_of("7 // 2;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(7))),
                op: BinaryOp::FloorDiv,
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_mod() {
        assert_eq!(
            expr_of("7 % 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(7))),
                op: BinaryOp::Mod,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    // -- unary -------------------------------------------------------

    #[test]
    fn test_expr_unary_minus() {
        assert_eq!(
            expr_of("-1;"),
            Expr::Unary {
                op: UnaryOp::Minus,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }
        );
    }

    #[test]
    fn test_expr_unary_plus() {
        assert_eq!(
            expr_of("+1;"),
            Expr::Unary {
                op: UnaryOp::Plus,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }
        );
    }

    #[test]
    fn test_expr_unary_bitnot() {
        assert_eq!(
            expr_of("~5;"),
            Expr::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(5))),
            }
        );
    }

    // -- power -------------------------------------------------------

    #[test]
    fn test_expr_power() {
        assert_eq!(
            expr_of("2 ** 3;"),
            Expr::Power {
                left: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    // -- bitwise -----------------------------------------------------

    #[test]
    fn test_expr_bitand() {
        assert_eq!(
            expr_of("5 & 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(5))),
                op: BinaryOp::BitAnd,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_bitor() {
        assert_eq!(
            expr_of("5 | 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(5))),
                op: BinaryOp::BitOr,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_bitxor() {
        assert_eq!(
            expr_of("5 ^ 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(5))),
                op: BinaryOp::BitXor,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    // -- shift -------------------------------------------------------

    #[test]
    fn test_expr_shl() {
        assert_eq!(
            expr_of("1 << 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                op: BinaryOp::Shl,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_shr() {
        assert_eq!(
            expr_of("8 >> 1;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(8))),
                op: BinaryOp::Shr,
                right: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }
        );
    }

    // -- comparison --------------------------------------------------

    #[test]
    fn test_expr_eq() {
        assert_eq!(
            expr_of("1 == 1;"),
            Expr::Compare {
                op: CompareOp::Eq,
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }
        );
    }

    #[test]
    fn test_expr_ne() {
        assert_eq!(
            expr_of("1 != 2;"),
            Expr::Compare {
                op: CompareOp::NotEq,
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_lt() {
        assert_eq!(
            expr_of("1 < 2;"),
            Expr::Compare {
                op: CompareOp::Lt,
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_le() {
        assert_eq!(
            expr_of("1 <= 2;"),
            Expr::Compare {
                op: CompareOp::Le,
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_gt() {
        assert_eq!(
            expr_of("3 > 2;"),
            Expr::Compare {
                op: CompareOp::Gt,
                left: Box::new(Expr::Literal(LiteralExpr::Int(3))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    #[test]
    fn test_expr_ge() {
        assert_eq!(
            expr_of("3 >= 2;"),
            Expr::Compare {
                op: CompareOp::Ge,
                left: Box::new(Expr::Literal(LiteralExpr::Int(3))),
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    // -- logical -----------------------------------------------------

    #[test]
    fn test_expr_and() {
        assert_eq!(
            expr_of("True and False;"),
            Expr::Logical {
                op: LogicalOp::And,
                left: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                right: Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
            }
        );
    }

    #[test]
    fn test_expr_or() {
        assert_eq!(
            expr_of("True or False;"),
            Expr::Logical {
                op: LogicalOp::Or,
                left: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                right: Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
            }
        );
    }

    #[test]
    fn test_expr_not() {
        assert_eq!(
            expr_of("not True;"),
            Expr::LogicalNot(Box::new(Expr::Literal(LiteralExpr::Boolean(true))))
        );
    }

    #[test]
    fn test_expr_double_not() {
        // `not not x` should produce nested LogicalNot nodes — the
        // grammar rule is `"not" not_expr`, so a second `not`
        // recurses rather than producing something flat.
        assert_eq!(
            expr_of("not not True;"),
            Expr::LogicalNot(Box::new(Expr::LogicalNot(Box::new(Expr::Literal(
                LiteralExpr::Boolean(true)
            )))))
        );
    }

    // -- ternary -----------------------------------------------------

    #[test]
    fn test_expr_ternary() {
        assert_eq!(
            expr_of("1 if True else 2;"),
            Expr::Ternary {
                true_expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                test: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                else_expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    // -- lambda ------------------------------------------------------

    #[test]
    fn test_expr_lambda_single_param() {
        assert_eq!(
            expr_of("lambda x: x + 1;"),
            Expr::Lambda {
                param_list: vec!["x".to_string()],
                expression: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Ident("x".to_string())),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                }),
            }
        );
    }

    #[test]
    fn test_expr_lambda_multi_params() {
        assert_eq!(
            expr_of("lambda a, b: a * b;"),
            Expr::Lambda {
                param_list: vec!["a".to_string(), "b".to_string()],
                expression: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Ident("a".to_string())),
                    op: BinaryOp::Mul,
                    right: Box::new(Expr::Ident("b".to_string())),
                }),
            }
        );
    }

    // -- grouping ----------------------------------------------------

    #[test]
    fn test_expr_grouped() {
        // `(1 + 2)` is just `1 + 2` — the parser flattens a single
        // parenthesized expression via `parse_tuple_or_grouped`.
        assert_eq!(
            expr_of("(1 + 2);"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                op: BinaryOp::Plus,
                right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            }
        );
    }

    // -- literals ----------------------------------------------------

    #[test]
    fn test_expr_list_literal() {
        assert_eq!(
            expr_of("[1, 2, 3];"),
            Expr::ListLiteral(vec![
                Expr::Literal(LiteralExpr::Int(1)),
                Expr::Literal(LiteralExpr::Int(2)),
                Expr::Literal(LiteralExpr::Int(3)),
            ])
        );
    }

    #[test]
    fn test_expr_tuple_literal() {
        assert_eq!(
            expr_of("(1, 2);"),
            Expr::TupleLiteral(vec![
                Expr::Literal(LiteralExpr::Int(1)),
                Expr::Literal(LiteralExpr::Int(2)),
            ])
        );
    }

    #[test]
    fn test_expr_dict_literal() {
        assert_eq!(
            expr_of(r#"{"a": 1};"#),
            Expr::DictLiteral(vec![crate::ast::DictEntry {
                key: Box::new(Expr::Literal(LiteralExpr::Str("a".to_string()))),
                value: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }])
        );
    }

    // -- precedence & associativity ----------------------------------

    #[test]
    fn test_expr_precedence_mul_over_add() {
        // `*` binds tighter than `+` → `1 + (2 * 3)`.
        assert_eq!(
            expr_of("1 + 2 * 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                op: BinaryOp::Plus,
                right: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                    op: BinaryOp::Mul,
                    right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
                }),
            }
        );
    }

    #[test]
    fn test_expr_left_assoc_sub() {
        // `-` is left-associative → `(1 - 2) - 3`.
        assert_eq!(
            expr_of("1 - 2 - 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                    op: BinaryOp::Minus,
                    right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                }),
                op: BinaryOp::Minus,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_power_right_assoc() {
        // `**` is right-associative → `2 ** (3 ** 2)`.
        assert_eq!(
            expr_of("2 ** 3 ** 2;"),
            Expr::Power {
                left: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                right: Box::new(Expr::Power {
                    left: Box::new(Expr::Literal(LiteralExpr::Int(3))),
                    right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                }),
            }
        );
    }

    #[test]
    fn test_expr_unary_binds_tighter_than_binary() {
        // `-2 + 3` → `(-2) + 3`.
        assert_eq!(
            expr_of("-2 + 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                }),
                op: BinaryOp::Plus,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_comparison_binds_looser_than_arith() {
        // `1 + 2 == 3` → `(1 + 2) == 3`.
        assert_eq!(
            expr_of("1 + 2 == 3;"),
            Expr::Compare {
                op: CompareOp::Eq,
                left: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                }),
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_not_binds_looser_than_comparison() {
        // In Python `not` is between `and` and comparisons, so
        // `1 == 1 or 2 == 2` first becomes `(1 == 1) or (2 == 2)`,
        // and `not True == False` parses as `not (True == False)`.
        assert_eq!(
            expr_of("not True == False;"),
            Expr::LogicalNot(Box::new(Expr::Compare {
                op: CompareOp::Eq,
                left: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                right: Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
            }))
        );
    }

    #[test]
    fn test_expr_and_binds_tighter_than_or() {
        // `a or b and c` → `a or (b and c)`.
        assert_eq!(
            expr_of("True or False and False;"),
            Expr::Logical {
                op: LogicalOp::Or,
                left: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                right: Box::new(Expr::Logical {
                    op: LogicalOp::And,
                    left: Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
                    right: Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
                }),
            }
        );
    }

    // -----------------------------------------------------------------
    // Boolean / None / Int / Float literals.
    //
    // The existing `test_parse_literal` / `test_parse_literal1` only
    // cover float and string at the top level. These tests pin the
    // remaining literal forms inside `expr_of` so the parser cannot
    // silently regress on them.
    // -----------------------------------------------------------------

    // -- literals (boolean / none) -----------------------------------

    #[test]
    fn test_expr_bool_true() {
        assert_eq!(expr_of("True;"), Expr::Literal(LiteralExpr::Boolean(true)));
    }

    #[test]
    fn test_expr_bool_false() {
        assert_eq!(
            expr_of("False;"),
            Expr::Literal(LiteralExpr::Boolean(false))
        );
    }

    #[test]
    fn test_expr_none() {
        assert_eq!(expr_of("None;"), Expr::Literal(LiteralExpr::None));
    }

    // -- literals (int / float) --------------------------------------

    #[test]
    fn test_expr_int_literal() {
        assert_eq!(expr_of("42;"), Expr::Literal(LiteralExpr::Int(42)));
    }

    #[test]
    fn test_expr_negative_int() {
        // `-42` is `Unary { op: Minus, expr: Literal(Int(42)) }`,
        // not a negative literal — the lexer only emits positive
        // integers and unary minus is the parser's job.
        assert_eq!(
            expr_of("-42;"),
            Expr::Unary {
                op: UnaryOp::Minus,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(42))),
            }
        );
    }

    #[test]
    fn test_expr_float_literal() {
        assert_eq!(expr_of("3.14;"), Expr::Literal(LiteralExpr::Float(3.14)));
    }

    // -----------------------------------------------------------------
    // Compound assignment operators.
    //
    // `=` and `+=` are covered by `test_parse_assign*`. The remaining
    // 11 compound operators are all matched by `is_assign_op` and
    // mapped to `AssignOperator::*` via `AssignOperator::from`. These
    // tests pin each one down so a future edit to the token list or
    // the `From<Token>` impl cannot silently drop one.
    // -----------------------------------------------------------------

    // -- compound assignment ------------------------------------------

    #[test]
    fn test_parse_assign_minus() {
        let program = parse_source("x -= 1;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::MinusAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_mul() {
        let program = parse_source("x *= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::MulAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_div() {
        let program = parse_source("x /= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::DivAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_floordiv() {
        let program = parse_source("x //= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::FloorAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_mod() {
        let program = parse_source("x %= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::ModAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_pow() {
        let program = parse_source("x **= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::PowAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_bitand() {
        let program = parse_source("x &= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::BitAndAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_bitor() {
        let program = parse_source("x |= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::BitOrAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_bitxor() {
        let program = parse_source("x ^= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::BitXorAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_shl() {
        let program = parse_source("x <<= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::ShlAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_shr() {
        let program = parse_source("x >>= 2;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Name("x".to_string()),
                op: AssignOperator::ShrAssign,
                expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
            })])
        );
    }

    #[test]
    fn test_parse_assign_tuple_target() {
        // `(a, b) = (1, 2);` — parenthesized tuple destructure on the
        // LHS, parenthesized tuple literal on the RHS. Both sides go
        // through `parse_tuple_or_grouped`; the LHS ends up as
        // `AssignTarget::Tuple` and the RHS as `Expr::TupleLiteral`.
        let program = parse_source("(a, b) = (1, 2);");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Assign {
                target: AssignTarget::Tuple(vec!["a".to_string(), "b".to_string()]),
                op: AssignOperator::Assign,
                expr: Box::new(Expr::TupleLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                ])),
            })])
        );
    }

    // -----------------------------------------------------------------
    // If statement shapes.
    //
    // The existing tests do not exercise `parse_if` at all. The block
    // bodies follow the parser's "no trailing `;` after a non-call
    // expression" rule — see the comment header on the `parse_block`
    // implementation for the rationale.
    // -----------------------------------------------------------------

    // -- if -----------------------------------------------------------

    #[test]
    fn test_parse_if_no_else() {
        let program = parse_source("if True { 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::If {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                then: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
                elif_branches: vec![],
                else_branch: None,
            })])
        );
    }

    #[test]
    fn test_parse_if_else() {
        let program = parse_source("if True { 1 } else { 2 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::If {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                then: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
                elif_branches: vec![],
                else_branch: Some(Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(2)
                )))])),
            })])
        );
    }

    #[test]
    fn test_parse_if_elif() {
        let program = parse_source("if True { 1 } elif False { 2 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::If {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                then: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
                elif_branches: vec![(
                    Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
                    Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                        LiteralExpr::Int(2)
                    )))])
                )],
                else_branch: None,
            })])
        );
    }

    #[test]
    fn test_parse_if_elif_else() {
        let program = parse_source("if True { 1 } elif False { 2 } else { 3 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::If {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                then: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
                elif_branches: vec![(
                    Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
                    Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                        LiteralExpr::Int(2)
                    )))])
                )],
                else_branch: Some(Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(3)
                )))])),
            })])
        );
    }

    #[test]
    fn test_parse_if_multiple_elifs() {
        let program = parse_source("if True { 1 } elif False { 2 } elif True { 3 } else { 4 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::If {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                then: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
                elif_branches: vec![
                    (
                        Box::new(Expr::Literal(LiteralExpr::Boolean(false))),
                        Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                            LiteralExpr::Int(2)
                        )))]),
                    ),
                    (
                        Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                        Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                            LiteralExpr::Int(3)
                        )))]),
                    ),
                ],
                else_branch: Some(Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(4)
                )))])),
            })])
        );
    }

    // -----------------------------------------------------------------
    // While / for / return.
    //
    // Bodies follow the same "no trailing `;` after a non-call
    // expression" rule. `return` and the loop forms are tested at the
    // top level (or inside a function body whose own `}` is consumed
    // by the enclosing `parse_block`).
    // -----------------------------------------------------------------

    // -- while / for --------------------------------------------------

    #[test]
    fn test_parse_while() {
        let program = parse_source("while True { 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::While {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                body: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
            })])
        );
    }

    #[test]
    fn test_parse_while_with_call_body() {
        // Function-call body is the one case where a trailing `;` is
        // OK inside a block — `parse_postfix` consumes it after the
        // closing `)`. This test pins that behavior.
        let program = parse_source("while True { print(1); }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::While {
                cond: Box::new(Expr::Literal(LiteralExpr::Boolean(true))),
                body: Block(vec![Stmt::ExprStmt(Box::new(Expr::FuncCall {
                    fn_expr: Box::new(Expr::Ident("print".to_string())),
                    args: vec![Expr::Literal(LiteralExpr::Int(1))],
                }))]),
            })])
        );
    }

    #[test]
    fn test_parse_for() {
        let program = parse_source("for x in items { 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::For {
                loop_var: "x".to_string(),
                iter_expr: Box::new(Expr::Ident("items".to_string())),
                body: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
            })])
        );
    }

    #[test]
    fn test_parse_for_list_iter() {
        // The iter expression is parsed via `parse_expression`, so
        // any expression form works — here, a list literal.
        let program = parse_source("for x in [1, 2, 3] { x }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::For {
                loop_var: "x".to_string(),
                iter_expr: Box::new(Expr::ListLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                    Expr::Literal(LiteralExpr::Int(3)),
                ])),
                body: Block(vec![Stmt::ExprStmt(Box::new(Expr::Ident("x".to_string())))]),
            })])
        );
    }

    // -- return -------------------------------------------------------

    #[test]
    fn test_parse_return_no_value() {
        // At the top level, the outer `parse` loop strips the
        // trailing `;`, so `return;` parses cleanly. Inside a block
        // body the same `;` would dangle and the next `parse_stmt`
        // would fail — that is exercised indirectly by the
        // function-body tests.
        let program = parse_source("return;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Return { value: None })])
        );
    }

    #[test]
    fn test_parse_return_value() {
        let program = parse_source("return 1;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Return {
                value: Some(Expr::Literal(LiteralExpr::Int(1))),
            })])
        );
    }

    #[test]
    fn test_parse_return_expr() {
        let program = parse_source("return a + b;");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Return {
                value: Some(Expr::BinaryExpr {
                    left: Box::new(Expr::Ident("a".to_string())),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Ident("b".to_string())),
                }),
            })])
        );
    }

    #[test]
    fn test_parse_return_inside_function() {
        // `return 1` (no `;` after) is the form that works inside a
        // function body — the body's `}` terminates the return and is
        // consumed by `parse_block`'s trailing advance.
        let program = parse_source("def m() { return 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Func {
                name: "m".to_string(),
                param_list: vec![],
                body: Block(vec![Stmt::Return {
                    value: Some(Expr::Literal(LiteralExpr::Int(1))),
                }]),
            })])
        );
    }

    // -----------------------------------------------------------------
    // Function with parameters.
    //
    // `parse_function` always sets `default: None` and never reads a
    // `= default` form, so we don't test default values here. These
    // tests pin the param-list parsing and the body shape.
    // -----------------------------------------------------------------

    // -- function with params -----------------------------------------

    #[test]
    fn test_parse_function_one_param() {
        let program = parse_source("def f(x) { x }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Func {
                name: "f".to_string(),
                param_list: vec![crate::ast::Parameter {
                    name: "x".to_string(),
                    default: None,
                }],
                body: Block(vec![Stmt::ExprStmt(Box::new(Expr::Ident("x".to_string())))]),
            })])
        );
    }

    #[test]
    fn test_parse_function_multi_params() {
        let program = parse_source("def f(a, b, c) { 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Func {
                name: "f".to_string(),
                param_list: vec![
                    crate::ast::Parameter {
                        name: "a".to_string(),
                        default: None,
                    },
                    crate::ast::Parameter {
                        name: "b".to_string(),
                        default: None,
                    },
                    crate::ast::Parameter {
                        name: "c".to_string(),
                        default: None,
                    },
                ],
                body: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                    LiteralExpr::Int(1)
                )))]),
            })])
        );
    }

    // -----------------------------------------------------------------
    // Empty block / empty dict.
    //
    // `parse_stmt` always tries `try_parse_dict_literal_at_stmt`
    // first, so a bare `{}` is an empty *dict* (ExprStmt), not a
    // block. A block with no statements can only appear as a function
    // body, where `parse_block` accepts the immediately-closing `}`.
    // -----------------------------------------------------------------

    // -- empty block / empty dict -------------------------------------

    #[test]
    fn test_parse_empty_block() {
        // `def f() {}` is the only way to get a `Stmt::Block` with
        // no statements at the top level: a bare `{}` is a dict.
        let program = parse_source("def f() {}");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Func {
                name: "f".to_string(),
                param_list: vec![],
                body: Block(vec![]),
            })])
        );
    }

    #[test]
    fn test_parse_empty_dict_expr() {
        // Bare `{}` is always parsed as an empty dict (not a block)
        // by `try_parse_dict_literal_at_stmt`. The trailing `;` is
        // consumed by the outer `parse` loop.
        let program = parse_source("{};");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::ExprStmt(Box::new(
                Expr::DictLiteral(Vec::new())
            )))])
        );
    }

    #[test]
    fn test_parse_block_with_one_stmt() {
        // `{ 1 }` is a block (not a dict) because the first inner
        // expression is not followed by `:`.
        let program = parse_source("{ 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Block(Block(vec![Stmt::ExprStmt(
                Box::new(Expr::Literal(LiteralExpr::Int(1)))
            )])))])
        );
    }

    // -----------------------------------------------------------------
    // Multi-statement programs.
    //
    // The outer `parse` loop (parser.rs:43-56) strips leading `;`
    // tokens, so multiple statements separated by `;` form a single
    // Program.
    // -----------------------------------------------------------------

    // -- multi-statement program --------------------------------------

    #[test]
    fn test_parse_multi_stmt_program() {
        let program = parse_source("x = 1; y = 2; z = x + y;");
        assert_eq!(
            program,
            Program(vec![
                Node::new(Stmt::Assign {
                    target: AssignTarget::Name("x".to_string()),
                    op: AssignOperator::Assign,
                    expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                }),
                Node::new(Stmt::Assign {
                    target: AssignTarget::Name("y".to_string()),
                    op: AssignOperator::Assign,
                    expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                }),
                Node::new(Stmt::Assign {
                    target: AssignTarget::Name("z".to_string()),
                    op: AssignOperator::Assign,
                    expr: Box::new(Expr::BinaryExpr {
                        left: Box::new(Expr::Ident("x".to_string())),
                        op: BinaryOp::Plus,
                        right: Box::new(Expr::Ident("y".to_string())),
                    }),
                }),
            ])
        );
    }

    #[test]
    fn test_parse_multi_stmt_mixed() {
        // Import + function + class. The class body uses a trailing
        // assignment (no `;`) so its `}` is consumed by
        // `parse_simple_assign`'s trailing advance — see the
        // `class` tests below for the detailed explanation.
        let program =
            parse_source("import foo; def g() { 1 } class C { def m() { print(1); } x = 1 }");
        assert_eq!(
            program,
            Program(vec![
                Node::new(Stmt::Import {
                    path: vec!["foo".to_string()],
                    alias: None,
                }),
                Node::new(Stmt::Func {
                    name: "g".to_string(),
                    param_list: vec![],
                    body: Block(vec![Stmt::ExprStmt(Box::new(Expr::Literal(
                        LiteralExpr::Int(1)
                    )))]),
                }),
                Node::new(Stmt::Class {
                    name: "C".to_string(),
                    members: vec![
                        ClassMember::FuncDecl(Stmt::Func {
                            name: "m".to_string(),
                            param_list: vec![],
                            body: Block(vec![Stmt::ExprStmt(Box::new(Expr::FuncCall {
                                fn_expr: Box::new(Expr::Ident("print".to_string())),
                                args: vec![Expr::Literal(LiteralExpr::Int(1))],
                            }))]),
                        }),
                        ClassMember::Assign(Stmt::Assign {
                            target: AssignTarget::Name("x".to_string()),
                            op: AssignOperator::Assign,
                            expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                        }),
                    ],
                }),
            ])
        );
    }

    // -----------------------------------------------------------------
    // Attribute / call chains.
    //
    // `parse_postfix` is a single-pass loop that re-enters after a
    // `(`, `[`, or `.` op, so chained accesses and calls fold into
    // nested `Attribute` / `FuncCall` / `Index` nodes.
    // -----------------------------------------------------------------

    // -- attribute / call chains --------------------------------------

    #[test]
    fn test_parse_attribute_chain() {
        // `a.b.c` left-folds into `Attribute(Attribute(a, b), c)`.
        assert_eq!(
            expr_of("a.b.c;"),
            Expr::Attribute {
                target: Box::new(Expr::Attribute {
                    target: Box::new(Expr::Ident("a".to_string())),
                    field_name: "b".to_string(),
                }),
                field_name: "c".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_attribute_after_call() {
        // `a().b` — call result fed into attribute access.
        assert_eq!(
            expr_of("a().b;"),
            Expr::Attribute {
                target: Box::new(Expr::FuncCall {
                    fn_expr: Box::new(Expr::Ident("a".to_string())),
                    args: vec![],
                }),
                field_name: "b".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_attribute_then_call() {
        // `a.b()` — attribute access, then call on the attribute.
        assert_eq!(
            expr_of("a.b();"),
            Expr::FuncCall {
                fn_expr: Box::new(Expr::Attribute {
                    target: Box::new(Expr::Ident("a".to_string())),
                    field_name: "b".to_string(),
                }),
                args: vec![],
            }
        );
    }

    #[test]
    fn test_parse_call_multi_args() {
        assert_eq!(
            expr_of("f(1, 2, 3);"),
            Expr::FuncCall {
                fn_expr: Box::new(Expr::Ident("f".to_string())),
                args: vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                    Expr::Literal(LiteralExpr::Int(3)),
                ],
            }
        );
    }

    #[test]
    fn test_parse_call_nested() {
        // `f(g(1), h(2))` — outer call's args are themselves calls.
        assert_eq!(
            expr_of("f(g(1), h(2));"),
            Expr::FuncCall {
                fn_expr: Box::new(Expr::Ident("f".to_string())),
                args: vec![
                    Expr::FuncCall {
                        fn_expr: Box::new(Expr::Ident("g".to_string())),
                        args: vec![Expr::Literal(LiteralExpr::Int(1))],
                    },
                    Expr::FuncCall {
                        fn_expr: Box::new(Expr::Ident("h".to_string())),
                        args: vec![Expr::Literal(LiteralExpr::Int(2))],
                    },
                ],
            }
        );
    }

    // -----------------------------------------------------------------
    // Nested / multi-operator expressions.
    // -----------------------------------------------------------------

    // -- nested expressions -------------------------------------------

    #[test]
    fn test_expr_nested_arith() {
        // `(1 + 2) * 3` — grouping is a syntactic form, the inner
        // expression is what the AST records.
        assert_eq!(
            expr_of("(1 + 2) * 3;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::BinaryExpr {
                    left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                    op: BinaryOp::Plus,
                    right: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                }),
                op: BinaryOp::Mul,
                right: Box::new(Expr::Literal(LiteralExpr::Int(3))),
            }
        );
    }

    #[test]
    fn test_expr_call_inside_arith() {
        assert_eq!(
            expr_of("f(1) + g(2);"),
            Expr::BinaryExpr {
                left: Box::new(Expr::FuncCall {
                    fn_expr: Box::new(Expr::Ident("f".to_string())),
                    args: vec![Expr::Literal(LiteralExpr::Int(1))],
                }),
                op: BinaryOp::Plus,
                right: Box::new(Expr::FuncCall {
                    fn_expr: Box::new(Expr::Ident("g".to_string())),
                    args: vec![Expr::Literal(LiteralExpr::Int(2))],
                }),
            }
        );
    }

    #[test]
    fn test_expr_index_inside_arith() {
        assert_eq!(
            expr_of("arr[0] + 1;"),
            Expr::BinaryExpr {
                left: Box::new(Expr::Index {
                    target: Box::new(Expr::Ident("arr".to_string())),
                    index: Box::new(Expr::Literal(LiteralExpr::Int(0))),
                }),
                op: BinaryOp::Plus,
                right: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }
        );
    }

    // -----------------------------------------------------------------
    // Tuple / list / dict shapes.
    //
    // The existing tests cover only `(1, 2)` and `{"a": 1}`. These
    // pin down the remaining shapes including the empty tuple, the
    // single-element tuple, trailing commas, and nested collections.
    // -----------------------------------------------------------------

    // -- tuple / list / dict shapes -----------------------------------

    #[test]
    fn test_expr_empty_tuple() {
        // `()` is the empty tuple per `parse_tuple_or_grouped`.
        assert_eq!(expr_of("();"), Expr::TupleLiteral(Vec::new()));
    }

    #[test]
    fn test_expr_single_element_tuple() {
        // `(1,)` is the one-element tuple — the trailing comma is
        // what distinguishes it from a grouped expression `(1)`.
        assert_eq!(
            expr_of("(1,);"),
            Expr::TupleLiteral(vec![Expr::Literal(LiteralExpr::Int(1))])
        );
    }

    #[test]
    fn test_expr_nested_list() {
        assert_eq!(
            expr_of("[[1, 2], [3, 4]];"),
            Expr::ListLiteral(vec![
                Expr::ListLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                ]),
                Expr::ListLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(3)),
                    Expr::Literal(LiteralExpr::Int(4)),
                ]),
            ])
        );
    }

    #[test]
    fn test_expr_nested_dict() {
        assert_eq!(
            expr_of(r#"{"a": {"b": 1}};"#),
            Expr::DictLiteral(vec![crate::ast::DictEntry {
                key: Box::new(Expr::Literal(LiteralExpr::Str("a".to_string()))),
                value: Box::new(Expr::DictLiteral(vec![crate::ast::DictEntry {
                    key: Box::new(Expr::Literal(LiteralExpr::Str("b".to_string()))),
                    value: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                }])),
            }])
        );
    }

    #[test]
    fn test_expr_dict_multi_entry() {
        assert_eq!(
            expr_of(r#"{"a": 1, "b": 2};"#),
            Expr::DictLiteral(vec![
                crate::ast::DictEntry {
                    key: Box::new(Expr::Literal(LiteralExpr::Str("a".to_string()))),
                    value: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                },
                crate::ast::DictEntry {
                    key: Box::new(Expr::Literal(LiteralExpr::Str("b".to_string()))),
                    value: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                },
            ])
        );
    }

    #[test]
    fn test_expr_dict_trailing_comma() {
        // `parse_dict_literal` allows a trailing `,` before `}`.
        assert_eq!(
            expr_of(r#"{"a": 1,}"#),
            Expr::DictLiteral(vec![crate::ast::DictEntry {
                key: Box::new(Expr::Literal(LiteralExpr::Str("a".to_string()))),
                value: Box::new(Expr::Literal(LiteralExpr::Int(1))),
            }])
        );
    }

    #[test]
    fn test_expr_list_trailing_comma() {
        // `parse_list_literal` allows a trailing `,` before `]`.
        assert_eq!(
            expr_of("[1, 2,];"),
            Expr::ListLiteral(vec![
                Expr::Literal(LiteralExpr::Int(1)),
                Expr::Literal(LiteralExpr::Int(2)),
            ])
        );
    }

    #[test]
    fn test_expr_nested_tuple() {
        assert_eq!(
            expr_of("((1, 2), (3, 4));"),
            Expr::TupleLiteral(vec![
                Expr::TupleLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                ]),
                Expr::TupleLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(3)),
                    Expr::Literal(LiteralExpr::Int(4)),
                ]),
            ])
        );
    }

    // -----------------------------------------------------------------
    // Class parsing.
    // -----------------------------------------------------------------

    // -- class --------------------------------------------------------

    #[test]
    fn test_parse_class_with_assignment() {
        // Single-assignment body — the trailing `self.advance()` in
        // `parse_simple_assign` consumes the class's `}`.
        let program = parse_source("class C { x = 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Class {
                name: "C".to_string(),
                members: vec![ClassMember::Assign(Stmt::Assign {
                    target: AssignTarget::Name("x".to_string()),
                    op: AssignOperator::Assign,
                    expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                })],
            })])
        );
    }

    #[test]
    fn test_parse_class_with_method() {
        // Method + trailing assignment. The method's body is a
        // function call (so `parse_postfix` consumes the `;`), and
        // the trailing assignment consumes the class's `}`.
        let program = parse_source("class C { def m() { print(1); } x = 1 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Class {
                name: "C".to_string(),
                members: vec![
                    ClassMember::FuncDecl(Stmt::Func {
                        name: "m".to_string(),
                        param_list: vec![],
                        body: Block(vec![Stmt::ExprStmt(Box::new(Expr::FuncCall {
                            fn_expr: Box::new(Expr::Ident("print".to_string())),
                            args: vec![Expr::Literal(LiteralExpr::Int(1))],
                        }))]),
                    }),
                    ClassMember::Assign(Stmt::Assign {
                        target: AssignTarget::Name("x".to_string()),
                        op: AssignOperator::Assign,
                        expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                    }),
                ],
            })])
        );
    }

    #[test]
    fn test_parse_class_with_multiple_members() {
        // Assignment (with `;`), method, assignment (no `;`).
        let program = parse_source("class C { x = 1; def m() { print(1); } y = 2 }");
        assert_eq!(
            program,
            Program(vec![Node::new(Stmt::Class {
                name: "C".to_string(),
                members: vec![
                    ClassMember::Assign(Stmt::Assign {
                        target: AssignTarget::Name("x".to_string()),
                        op: AssignOperator::Assign,
                        expr: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                    }),
                    ClassMember::FuncDecl(Stmt::Func {
                        name: "m".to_string(),
                        param_list: vec![],
                        body: Block(vec![Stmt::ExprStmt(Box::new(Expr::FuncCall {
                            fn_expr: Box::new(Expr::Ident("print".to_string())),
                            args: vec![Expr::Literal(LiteralExpr::Int(1))],
                        }))]),
                    }),
                    ClassMember::Assign(Stmt::Assign {
                        target: AssignTarget::Name("y".to_string()),
                        op: AssignOperator::Assign,
                        expr: Box::new(Expr::Literal(LiteralExpr::Int(2))),
                    }),
                ],
            })])
        );
    }

    // -----------------------------------------------------------------
    // Set literals — expected to FAIL.
    //
    // `Expr::SetLiteral` is defined in `ast.rs` and the grammar
    // (`docs/grammar.ebnf:293-297`) describes the `{x, y, z}` form,
    // but `parse_dict_literal` and `try_parse_dict_literal_at_stmt`
    // only produce `Expr::DictLiteral`. There is no set-literal
    // branch. These tests assert the expected set AST and will fail
    // with a `ParserError` from the missing-`:` check.
    //
    // See `docs/parser_bugs_01.md` for the recorded failures.
    // -----------------------------------------------------------------

    // -- set literals (expected failures) -----------------------------

    #[test]
    fn test_expr_set_literal() {
        // `{1, 2, 3}` — the comma after the first element signals a
        // set, not a dict (a dict requires `:`). The parser instead
        // routes this through `parse_block` and chokes on the
        // dangling `,`.
        assert_eq!(
            expr_of("{1, 2, 3};"),
            Expr::SetLiteral(vec![
                Expr::Literal(LiteralExpr::Int(1)),
                Expr::Literal(LiteralExpr::Int(2)),
                Expr::Literal(LiteralExpr::Int(3)),
            ])
        );
    }

    #[test]
    fn test_expr_set_literal_single() {
        assert_eq!(
            expr_of("{1};"),
            Expr::SetLiteral(vec![Expr::Literal(LiteralExpr::Int(1))])
        );
    }

    #[test]
    fn test_expr_set_literal_mixed() {
        assert_eq!(
            expr_of(r#"{1, "x", True};"#),
            Expr::SetLiteral(vec![
                Expr::Literal(LiteralExpr::Int(1)),
                Expr::Literal(LiteralExpr::Str("x".to_string())),
                Expr::Literal(LiteralExpr::Boolean(true)),
            ])
        );
    }

    // -----------------------------------------------------------------
    // Comparison operators `in` / `not in` / `is` / `is not` —
    // expected to FAIL.
    //
    // `CompareOp::In`, `CompareOp::NotIn`, `CompareOp::Is`, and
    // `CompareOp::IsNot` are defined in `ast.rs` but
    // `parse_comparison` (parser.rs:556-586) only matches
    // `==`, `!=`, `<`, `<=`, `>`, `>=`. The `in` / `is` keyword
    // tokens are never consumed inside `parse_comparison`, so the
    // surrounding `parse_assignment_stmt` falls through and the
    // outer parser then chokes on the leftover keyword.
    //
    // See `docs/parser_bugs_01.md` for the recorded failures.
    // -----------------------------------------------------------------

    // -- comparison in / is (expected failures) -----------------------

    #[test]
    fn test_expr_in() {
        assert_eq!(
            expr_of("1 in [1, 2];"),
            Expr::Compare {
                op: CompareOp::In,
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                right: Box::new(Expr::ListLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                ])),
            }
        );
    }

    #[test]
    fn test_expr_not_in() {
        assert_eq!(
            expr_of("1 not in [1, 2];"),
            Expr::Compare {
                op: CompareOp::NotIn,
                left: Box::new(Expr::Literal(LiteralExpr::Int(1))),
                right: Box::new(Expr::ListLiteral(vec![
                    Expr::Literal(LiteralExpr::Int(1)),
                    Expr::Literal(LiteralExpr::Int(2)),
                ])),
            }
        );
    }

    #[test]
    fn test_expr_is() {
        assert_eq!(
            expr_of("x is None;"),
            Expr::Compare {
                op: CompareOp::Is,
                left: Box::new(Expr::Ident("x".to_string())),
                right: Box::new(Expr::Literal(LiteralExpr::None)),
            }
        );
    }

    #[test]
    fn test_expr_is_not() {
        assert_eq!(
            expr_of("x is not None;"),
            Expr::Compare {
                op: CompareOp::IsNot,
                left: Box::new(Expr::Ident("x".to_string())),
                right: Box::new(Expr::Literal(LiteralExpr::None)),
            }
        );
    }

    #[test]
    fn test_parse_class_empty() {
        assert!(try_parse("class C {}").is_ok());
    }
}
