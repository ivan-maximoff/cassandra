use std::collections::HashMap;

use crate::{
    utils::{
        errors::Errors,
        token_conversor::{get_identifier_string, get_literal},
    }, parsers::tokens::{literal::Literal, token::Token, terms::ComparisonOperators},
};
use WhereClause::*;

use super::{comparison::ComparisonExpr, evaluate::Evaluate};

/// Enum para representar diferentes tipos de expresiones booleanas.
#[derive(Debug, PartialEq)]
pub enum WhereClause {
    Comparison(ComparisonExpr),
    Tuple(Vec<ComparisonExpr>),
    And(Box<WhereClause>, Box<WhereClause>),
    Or(Box<WhereClause>, Box<WhereClause>),
    Not(Box<WhereClause>),
}

impl Evaluate for WhereClause {
    /// Evalúa una expresión booleana usando los valores Columna -> Valor de una fila
    fn evaluate(&self, row: &HashMap<String, Literal>) -> Result<bool, Errors> {
        match self {
            Comparison(comparacion) => comparacion.evaluate(row),
            Tuple(comparaciones) => {
                for comparacion in comparaciones {
                    match comparacion.evaluate(row) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(err) => return Err(err),
                    }
                }
                Ok(true)
            }
            And(expr1, expr2) => Ok(expr1.evaluate(row)? && expr2.evaluate(row)?),
            Or(expr1, expr2) => Ok(expr1.evaluate(row)? || expr2.evaluate(row)?),
            Not(expr) => Ok(!expr.evaluate(row)?),
        }
    }
}

pub fn comparison_expr(
    column: &str,
    operator: ComparisonOperators,
    literal: Literal,
) -> WhereClause {
    Comparison(ComparisonExpr::new(column.to_string(), &operator, literal))
}

pub fn tuple_expr(exprs: Vec<ComparisonExpr>) -> WhereClause {
    Tuple(exprs)
}

pub fn build_tuple(
    column_names: Vec<Token>,
    literals: Vec<Token>,
    operator: ComparisonOperators,
) -> Result<WhereClause, Errors> {
    let iterations = column_names.len();
    let mut column_iter = column_names.into_iter().peekable();
    let mut literal_iter = literals.into_iter().peekable();

    let mut tuple = Vec::new();
    for _ in 0..iterations {
        let column_name = get_identifier_string(&mut column_iter)?;
        let literal = get_literal(&mut literal_iter)?;

        let expression = ComparisonExpr::new(column_name, &operator, literal);

        tuple.push(expression);
    }
    Ok(tuple_expr(tuple))
}

pub fn and_expr(left: WhereClause, right: WhereClause) -> WhereClause {
    And(Box::new(left), Box::new(right))
}

pub fn or_expr(left: WhereClause, right: WhereClause) -> WhereClause {
    Or(Box::new(left), Box::new(right))
}

pub fn not_expr(expr: WhereClause) -> WhereClause {
    Not(Box::new(expr))
}

#[cfg(test)]
mod tests {
    use crate::parsers::query_parsers::where_clause_::{
            comparison::ComparisonExpr, evaluate::Evaluate, where_clause::WhereClause,
        };
    use crate::parsers::tokens::terms::ComparisonOperators;
    use crate::parsers::tokens::literal::Literal;
    use crate::parsers::tokens::data_type::DataType;
    
    use std::collections::HashMap;
    use ComparisonOperators::*;
    use DataType::*;

    use super::{and_expr, comparison_expr, not_expr, or_expr, tuple_expr};

    fn assert_evaluation(row: HashMap<String, Literal>, clause: WhereClause, expected: bool) {
        match clause.evaluate(&row) {
            Ok(result) => assert_eq!(result, expected),
            Err(err) => panic!("Error test: {:?}", err),
        }
    }

    fn setup_row() -> HashMap<String, Literal> {
        let mut row = HashMap::new();
        row.insert("id".to_string(), Literal::new("5".to_string(), Int));
        row.insert("age".to_string(), Literal::new("30".to_string(), Int));
        row.insert("is_active".to_string(), Literal::new("true".to_string(), Boolean));
        row
    }

    #[test]
    fn test_comparison_true() {
        let row = setup_row();
        let clause = comparison_expr("id", Equal, Literal::new("5".to_string(), Int));
        assert_evaluation(row, clause, true);
    }

    #[test]
    fn test_comparison_false() {
        let row = setup_row();
        let clause = comparison_expr("id", Equal, Literal::new("10".to_string(), Int));
        assert_evaluation(row, clause, false);
    }

    #[test]
    fn test_tuple_true() {
        let row = setup_row();
        let clause = tuple_expr(vec![
            ComparisonExpr::new("id".to_string(), &Equal, Literal::new("5".to_string(), Int)),
            ComparisonExpr::new("age".to_string(), &Equal, Literal::new("30".to_string(), Int)),
        ]);
        assert_evaluation(row, clause, true);
    }

    #[test]
    fn test_tuple_false() {
        let row = setup_row();
        let clause = tuple_expr(vec![
            ComparisonExpr::new("id".to_string(), &Equal, Literal::new("5".to_string(), Int)),
            ComparisonExpr::new("age".to_string(), &Equal, Literal::new("40".to_string(), Int)),
        ]);
        assert_evaluation(row, clause, false);
    }

    #[test]
    fn test_and_true() {
        let row = setup_row();
        let clause = and_expr(
            comparison_expr("id", Equal, Literal::new("5".to_string(), Int)),
            comparison_expr("age", Equal, Literal::new("30".to_string(), Int)),
        );
        assert_evaluation(row, clause, true);
    }

    #[test]
    fn test_and_false() {
        let row = setup_row();
        let clause = and_expr(
            comparison_expr("id", Equal, Literal::new("5".to_string(), Int)),
            comparison_expr("age", Equal, Literal::new("40".to_string(), Int)),
        );
        assert_evaluation(row, clause, false);
    }

    #[test]
    fn test_or_true() {
        let row = setup_row();
        let clause = or_expr(
            comparison_expr("id", Equal, Literal::new("5".to_string(), Int)),
            comparison_expr("age", Equal, Literal::new("40".to_string(), Int)),
        );
        assert_evaluation(row, clause, true);
    }

    #[test]
    fn test_or_false() {
        let row = setup_row();
        let clause = or_expr(
            comparison_expr("id", Equal, Literal::new("10".to_string(), Int)),
            comparison_expr("age", Equal, Literal::new("40".to_string(), Int)),
        );
        assert_evaluation(row, clause, false);
    }

    #[test]
    fn test_not_true() {
        let row = setup_row();
        let clause = not_expr(comparison_expr(
            "is_active",
            Equal,
            Literal::new("false".to_string(), Boolean),
        ));
        assert_evaluation(row, clause, true);
    }

    #[test]
    fn test_not_false() {
        let row = setup_row();
        let clause = not_expr(comparison_expr(
            "is_active",
            Equal,
            Literal::new("true".to_string(), Boolean),
        ));
        assert_evaluation(row, clause, false);
    }
}
