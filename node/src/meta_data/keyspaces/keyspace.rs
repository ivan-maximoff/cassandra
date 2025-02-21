use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::table::Table;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Keyspace {
    pub tables: HashMap<String, Table>,
    pub replication_strategy: String,
    pub replication_factor: usize,
}

impl Keyspace {
    pub fn new(
        replication_strategy: Option<String>, // Puede recibir la estrategia o no
        replication_factor: Option<usize>,    // Puede recibir el factor o no
    ) -> Keyspace {
        let strategy = replication_strategy.unwrap_or("Simple Stategy".to_string());
        let factor = replication_factor.unwrap_or(3);
        Keyspace {
            tables: HashMap::new(), // Inicializamos el HashMap de tablas vacío
            replication_strategy: strategy,
            replication_factor: factor,
        }
    }

    pub fn set_replication_strategy(&mut self, strategy: String) {
        self.replication_strategy = strategy;
    }

    pub fn set_replication_factor(&mut self, factor: usize) {
        self.replication_factor = factor;
    }
}
