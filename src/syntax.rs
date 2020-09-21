use std::fmt::{self, Display};

use crate::tokens::Token;

/// A program is 1 or more statements.
#[derive(Debug)]
struct Program {
    statements: Vec<Stmt>,
}

#[doc(hidden)]
type BoxExpr = Box<Expr>;

/// Statements perform side effects.
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Assign a value to a variable.
    Assignment(Token, BoxExpr),
    /// Store a value in a register.
    Store(BoxExpr, BoxExpr),
    /// Resume program execution on the line indicated.
    Goto(BoxExpr),
    /// A normal assertion. Accepts `true` (1) and `false` (0).
    Assert(BoxExpr),
    /// An if statement. Accepts `true` (1) and `false` (0).
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
                format!("If {} Then Goto {} Else Goto {}", cond, iftrue, iffalse)
            }
        };

        write!(f, "{}", val)
    }
}

/// Expressions evaluate to values.
#[derive(Debug, Clone)]
pub enum Expr {
    /// Load a value from a registry stored by `Stmt::Store`.
    Load(BoxExpr),
    /// A binary operator, e.g. `+`.
    Binary(BoxExpr, Token, BoxExpr),
    /// A unary operator, such as `!`.
    Unary(Token, BoxExpr),
    /// A variable.
    Var(String),
    /// Load a value from some source, such as `stdin`.
    GetInput(String),
    /// A value. All simpIL values are 32-bit unsigned integers.
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
