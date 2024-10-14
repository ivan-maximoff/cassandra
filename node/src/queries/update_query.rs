use std::collections::HashMap;

use crate::utils::errors::Errors;

use super::{if_clause::IfClause, query::Query, set_logic::assigmente_value::AssignmentValue, where_logic::where_clause::WhereClause};

#[derive(PartialEq, Debug)]
pub struct UpdateQuery {
    pub table: String,
    pub changes: HashMap<String, AssignmentValue>,
    pub where_clause: Option<WhereClause>,
    pub if_clause: Option<IfClause>
}

impl UpdateQuery {
    pub fn new() -> Self {
        Self {
            table: String::new(),
            changes: HashMap::new(),
            where_clause: None,
            if_clause: None
        }
    }
}

impl Default for UpdateQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl Query for UpdateQuery {
    fn run(&self) -> Result<(), Errors> {
        todo!()
    }
}