use crate::{
    parsers::tokens::{literal::Literal, terms::ComparisonOperators},
    queries::{evaluate::Evaluate, where_logic::comparison::ComparisonExpr},
    utils::errors::Errors,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum IfClause {
    Exist,
    Comparison(ComparisonExpr),
    And(Box<IfClause>, Box<IfClause>),
    Or(Box<IfClause>, Box<IfClause>),
    Not(Box<IfClause>),
}

use IfClause::*;

impl Evaluate for IfClause {
    fn evaluate(&self, row: &HashMap<String, Literal>) -> Result<bool, Errors> {
        match self {
            Exist => Ok(!row.is_empty()), // if row exists because the where clause already checked the primary key
            Comparison(comparison) => comparison.evaluate(row),
            And(expr1, expr2) => Ok(expr1.evaluate(row)? && expr2.evaluate(row)?),
            Or(expr1, expr2) => Ok(expr1.evaluate(row)? || expr2.evaluate(row)?),
            Not(expr) => Ok(!expr.evaluate(row)?),
        }
    }
}

pub fn comparison_if(column: &str, operator: ComparisonOperators, literal: Literal) -> IfClause {
    Comparison(ComparisonExpr::new(column.to_string(), &operator, literal))
}

pub fn and_if(left: IfClause, right: IfClause) -> IfClause {
    And(Box::new(left), Box::new(right))
}

pub fn or_if(left: IfClause, right: IfClause) -> IfClause {
    Or(Box::new(left), Box::new(right))
}

pub fn not_if(expr: IfClause) -> IfClause {
    Not(Box::new(expr))
}
