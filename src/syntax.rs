use std::fmt::{self, Display};

use crate::tokens::Token;

#[derive(Debug)]
struct Program {
    statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment(Token, BoxExpr),
    Store(BoxExpr, BoxExpr),
    Goto(BoxExpr),
    Assert(BoxExpr),
    IfThenElse(BoxExpr, BoxExpr, BoxExpr),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self.clone() {
            Stmt::Assignment(var, expr) => format!("{} := {}", var, expr),
            Stmt::Store(lhs, rhs) => format!("Store({}, {})", lhs, rhs),
            Stmt::Goto(statement) => format!("Goto {}", statement),
            Stmt::Assert(expr) => format!("Assert {}", expr),
            Stmt::IfThenElse(cond, iftrue, iffalse) => {
                format!("if {} then {} else {}", cond, iftrue, iffalse)
            }
        };

        write!(f, "{}", val)
    }
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

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self.clone() {
            Expr::Load(reg) => format!("Load({})", reg),
            Expr::Binary(lhs, op, rhs) => format!("({}, {}, {})", lhs, op, rhs),
            Expr::Unary(op, rhs) => format!("Unary({}, {})", op, rhs),
            Expr::Var(var) => format!("{}", var),
            Expr::GetInput(input) => format!("GetInput({})", input),
            Expr::Val(val) => format!("{}", val),
        };

        write!(f, "{}", val)
    }
}
