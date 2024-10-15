use std::any::Any;
use crate::{queries::query::Query, utils::errors::Errors};

#[derive(PartialEq, Debug)]
pub struct DropKeySpaceQuery {
    pub keyspace: String,
    pub if_exist: Option<bool>,

}


impl DropKeySpaceQuery {
    pub fn new() -> Self {
        Self {
            keyspace: String::new(),
            if_exist: None,
        }
    }
}

impl Query for DropKeySpaceQuery {
    fn run(&self) -> Result<String, Errors> {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for DropKeySpaceQuery {
    fn default() -> Self {
         Self::new()
    }
}
