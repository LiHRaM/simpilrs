use crate::tokens::Token;

#[derive(Debug)]
struct Program {
    statements: Vec<Stmt>
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment(Token, BoxExpr),
    Store(BoxExpr, BoxExpr),
    Goto(BoxExpr),
    Assert(BoxExpr),
    IfThenElse(BoxExpr, BoxExpr, BoxExpr),
}

type BoxExpr = Box<Expr>;
#[derive(Debug, Clone)]
pub enum Expr {
    Load(BoxExpr),
    Binary(BoxExpr, Token, BoxExpr),
    Unary(Token, BoxExpr),
    Var(String),
    GetInput(String),
    Val(u32),
}