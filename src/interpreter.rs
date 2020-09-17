use crate::syntax::{Expr, Stmt};
use std::collections::HashMap as Map;
use tracing::event;
use tracing::Level;

pub struct Interpreter {
    statements: Vec<Stmt>,    // Sigma
    registers: Map<u32, u32>, // µ
    vars: Map<String, u32>,   // Delta
    program_counter: usize,   // pc
}

impl Interpreter {
    pub fn visit(mut self) -> Vec<u32> {
        let mut res = Vec::new();
        while self.program_counter < self.statements.len() {
            event!(Level::INFO, "Statement: {}", &self.program_counter);
            let statement = { self.statements[self.program_counter].clone() };
            res.push(self.visit_stmt(&statement));
        }
        res
    }

    pub fn new(statements: Vec<Stmt>) -> Self {
        Self {
            statements,
            registers: Map::new(),
            vars: Map::new(),
            program_counter: 0,
        }
    }
}

impl Interpreter {
    fn visit_stmt(&mut self, s: &Stmt) -> u32 {
        self.program_counter += 1;
        match s {
            Stmt::Assignment(identifier, expr) => {
                let expr = self.visit_expr(expr);
                self.vars.insert(identifier.lexeme.clone(), expr).unwrap()
            }
            Stmt::Store(reg, val) => {
                let reg = self.visit_expr(reg);
                let val = self.visit_expr(val);
                self.registers.insert(reg, val);
                val
            }
            Stmt::Goto(e) => {
                let e = self.visit_expr(e);
                self.program_counter = e as usize;
                e
            }
            Stmt::Assert(e) => {
                let e = self.visit_expr(e);
                if e == 1 {
                    e
                } else {
                    println!("Assert failed tho, sry.");
                    std::process::exit(1337);
                }
            }
            Stmt::IfThenElse(cond, lhs, rhs) => {
                let cond = self.visit_expr(cond);
                if cond == 1 {
                    self.visit_expr(lhs)
                } else if cond == 0 {
                    self.visit_expr(rhs)
                } else {
                    0
                }
            }
        }
    }

    fn visit_expr(&mut self, e: &Expr) -> u32 {
        match e {
            Expr::Load(expr) => {
                let expr = self.visit_expr(expr);
                self.registers.get(&expr).unwrap().to_owned()
            }
            Expr::Binary(lhs, op, rhs) => {
                let lhs = self.visit_expr(lhs);
                let rhs = self.visit_expr(rhs);
                match &op.token_type {
                    crate::tokens::TokenType::Plus => lhs + rhs,
                    crate::tokens::TokenType::Minus => lhs - rhs,
                    crate::tokens::TokenType::Star => lhs * rhs,
                    crate::tokens::TokenType::Slash => lhs / rhs,
                    t => panic!("Invalid binary token: {:#?}", t),
                }
            }
            Expr::Unary(_, expr) => {
                let expr = self.visit_expr(expr);
                expr
            }
            Expr::Var(identifier) => self.vars.get(identifier).unwrap().clone(),
            Expr::GetInput(_) => {
                let mut buffer = String::new();
                use std::io::{self, Read};
                let stdin = io::stdin();
                let mut handle = stdin.lock();

                handle.read_to_string(&mut buffer).unwrap();

                let val: u32 = buffer.parse().unwrap();
                val
            }
            Expr::Val(v) => v.clone(),
        }
    }
}
