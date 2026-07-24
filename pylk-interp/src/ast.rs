/// an ast node
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Stmt(Stmt),
    Expr(Expr),
}

/// Stmt is the action that control the flow of program
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt(Box<Expr>),
    Block(Vec<Stmt>),
    Import {
        pkg_name: String,
    },
    Func {
        name: String,
        param_list: Vec<Expr>,
        body: Box<Stmt>,
        ret: Expr,
    },
    Class {
        name: String,
        funcs: Vec<Stmt>,
        members: Vec<Stmt>,
    },

    If {
        cond: Box<Expr>,
        then: Box<Stmt>,
        elif_cond: Option<Box<Expr>>,
        elif_block: Option<Vec<Stmt>>,
        else_branch: Option<Box<Stmt>>,
    },

    While {
        cond: Option<Box<Expr>>,
        body: Box<Stmt>,
    },

    For {
        loop_var: String,
        in_expr: Box<Expr>,
        body: Box<Stmt>,
    },

    Return {
        expr: Option<Box<Expr>>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOperator {
    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    PowAssign,
    FloorAssign,
    MatMulAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    NotEq,
    Plus,
    Minus,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Power,
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
    Attribute(String),
    /// a[i] = 1, b['x'] = 'y'
    Indx {
        name: String,
        idx: Box<Expr>,
    },
}

/// Stmt is the computation that return a value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lambda {
        args: Vec<String>,
        body: Box<Stmt>,
    },
    Assign {
        target: AssignTarget,
        op: AssignOperator,
        expr: Box<Expr>,
    },
    Attribute {
        class_name: String,
        field_name: String,
    },
    FuncCall {
        fn_name: String,
        args: Vec<Expr>,
    },

    /// var = x if xxx else y
    Ternary {
        true_expr: Box<Expr>,
        test: Box<Expr>,
        else_expr: Box<Expr>,
    },
    LogicalOr {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    LogicalAnd {
        left: Box<Expr>,
        right: Box<Expr>,
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
    Ident(String),
    Literal(LiteralExpr),
    ListLiteral(Vec<Expr>),
    DictLiteral(Vec<Expr>),
    DictEntry {
        key: Box<Expr>,
        value: Box<Expr>,
    },
    TupleLiteral(Vec<Expr>),
    SetLiteral(Vec<Expr>),
    /// a[1:6:2]
    Slice {
        name: String,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
    },
}
