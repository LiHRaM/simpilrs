use crate::syntax::*;

pub trait Visitor<T> {
    fn visit_stmt(&mut self, s: &Stmt) -> T;
    fn visit_expr(&mut self, e: &Expr) -> T;
}