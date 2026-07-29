use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Program(pub Vec<Node>);

/// an ast node
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub stmt: Stmt,
    pub line: usize,
    pub col: usize,
}

impl Node {
    pub fn new(stmt: Stmt) -> Self {
        Self {
            stmt,
            line: 0,
            col: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub default: Option<Expr>,
}

/// One entry on the right-hand side of a `from ... import ...` list.
///
/// `name` is the original identifier; `alias` is the local binding name
/// when `as` is used. For a star-import (`from x import *`), the parser
/// produces a single `ImportName` with `name == "*"` and `alias == None`.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportName {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block(pub Vec<Stmt>);

/// Stmt is the action that control the flow of program
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt(Box<Expr>),
    Block(Block),
    Func {
        name: String,
        param_list: Vec<Parameter>,
        body: Block,
    },
    Class {
        name: String,
        members: Vec<Stmt>, // Vec<Funcs | ExprStmt>
    },

    /// import a.b.c as x
    Import {
        path: Vec<String>,
        alias: Option<String>,
    },

    /// from a.b.c import d, e as f
    /// `from a.b.c import *` is represented by `names` containing a
    /// single [`ImportName`] whose `name` is the literal string "*".
    FromImport {
        module: Vec<String>,
        names: Vec<ImportName>,
    },

    /*
    if cond
    elif cond1 {
    } elif cond2 {
     }
     else {
     }
    */
    If {
        cond: Box<Expr>,
        then: Block,
        elif_branches: Vec<(Box<Expr>, Block)>,
        else_branch: Option<Block>,
    },

    While {
        cond: Box<Expr>,
        body: Block,
    },

    For {
        loop_var: String,
        iter_expr: Box<Expr>,
        body: Block,
    },

    Return {
        value: Option<Expr>,
    },
    Break,
    Continue,
    Pass,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOperator {
    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowAssign, // **=
    FloorAssign,
    MatMulAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

impl From<Token> for AssignOperator {
    fn from(value: Token) -> Self {
        match value {
            Token::Assign => Self::Assign,
            Token::PlusAssign => Self::PlusAssign,
            Token::MinusAssign => Self::MinusAssign,
            Token::MulAssign => Self::MulAssign,
            Token::DivAssign => Self::DivAssign,
            Token::ModAssign => Self::ModAssign,
            Token::PowAssign => Self::PowAssign,
            Token::FloorAssign => Self::FloorAssign,
            Token::MatMulAssign => Self::MatMulAssign,
            Token::BitAndAssign => Self::BitAndAssign,
            Token::BitOrAssign => Self::BitOrAssign,
            Token::BitXorAssign => Self::BitXorAssign,
            Token::ShlAssign => Self::ShlAssign,
            Token::ShrAssign => Self::ShrAssign,
            // `:=` is conceptually an assignment; map it to plain Assign.
            Token::Walrus => Self::Assign,
            // Every other Token has no corresponding AssignOperator, so fall
            // back to plain assignment rather than panic.
            _ => Self::Assign,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mul,
    Div,
    FloorDiv,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl, // <<
    Shr, // >>
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Plus,   // +
    Minus,  // -
    BitNot, // ~
    Not,    // not
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Lt,    // <
    Gt,    // >
    Le,    // <=
    Ge,    // >=
    Eq,    // ==
    NotEq, // !=
    In,
    Is,
    NotIn,
    IsNot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExpr {
    Str(String),
    Int(i64),
    Float(f64),
    Boolean(bool),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    Name(String),
    Tuple(Vec<String>),
    /// obj.x = 123
    Attribute {
        obj: String,
        attr: String,
    },
    /// a[i] = 1, b['x'] = 'y'
    Indx {
        name: String,
        idx: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DictEntry {
    pub key: Box<Expr>,
    pub value: Box<Expr>,
}

/// Expr is the computation that return a value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lambda {
        param_list: Vec<String>,
        expression: Box<Expr>,
    },

    Assign {
        target: AssignTarget,
        op: AssignOperator,
        expr: Box<Expr>,
    },
    Attribute {
        target: Box<Expr>,
        field_name: String,
    },
    FuncCall {
        fn_expr: Box<Expr>,
        args: Vec<Expr>,
    },

    /// arr[1], users['user001']
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },

    /// var = x if xxx else y
    Ternary {
        true_expr: Box<Expr>,
        test: Box<Expr>,
        else_expr: Box<Expr>,
    },
    BinaryExpr {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Power {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Logical {
        op: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Compare {
        op: CompareOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Ident(String),
    Literal(LiteralExpr),
    ListLiteral(Vec<Expr>),
    DictLiteral(Vec<DictEntry>),
    TupleLiteral(Vec<Expr>),
    SetLiteral(Vec<Expr>),
    Group(Box<Expr>),
    
    /// a[1:6:2]
    Slice {
        name: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
}
