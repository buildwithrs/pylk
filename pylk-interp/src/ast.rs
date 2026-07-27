#[derive(Debug, Clone, PartialEq)]
pub struct Program(Vec<Node>);

/// an ast node
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub stmt: Stmt,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    name: String,
    default: Option<Expr>,
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
        alias: Option<String>
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
        elif_branches: Vec<(Box<Expr>, Option<Block>)>,
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
    Or
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Lt, // <
    Gt, // >
    Le, // <=
    Ge, // >=
    Eq, // ==
    NotEq, // !=
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

#[derive(Debug, Clone, PartialEq)]
pub struct DictEntry {
    key: Box<Expr>,
    value: Box<Expr>,
}

/// Stmt is the computation that return a value
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lambda {
        param_list: Vec<String>,
        body: Box<Expr>,
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
        right: Box<Expr>
    },
    Logical {
        op: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>
    },
    Compare {
        op: CompareOp,
        left: Box<Expr>,
        right: Box<Expr>
    },

    Ident(String),
    Literal(LiteralExpr),
    ListLiteral(Vec<Expr>),
    DictLiteral(Vec<DictEntry>),
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
