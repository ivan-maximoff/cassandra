use std::collections::HashSet;

const PALABRAS_RESERVADAS: &[&str] = &[
    "SELECT",
    "INSERT",
    "ALTER",
    "AND",
    "ASC",
    "AS",
    "BATCH",
    "BY",
    "CREATE",
    "DELETE",
    "DESC",
    "DISTINCT",
    "DROP",
    "FROM",
    "IF",
    "INTO",
    "KEY",
    "KEYS",
    "KEYSPACE",
    "KEYSPACES",
    "NOT",
    "NULL",
    "OR",
    "PRIMARY",
    "RENAME",
    "REPLACE",
    "SET",
    "TABLE",
    "TO",
    "TRUNCATE",
    "UPDATE",
    "USE",
    "USING",
    "VALUES",
    "WHERE",
    "WITH",
    "ORDER",
];

pub struct WordsReserved {
    words: HashSet<&'static str>,
}

impl WordsReserved {
    pub fn new() -> Self {
        let mut set = HashSet::new();
        for &word in PALABRAS_RESERVADAS {
            set.insert(word);
        }
        WordsReserved { words: set }
    }

    pub fn is_reserved(&self, word: &str) -> bool {
        self.words.contains(&word.to_uppercase().as_str())
    }
}

impl Default for WordsReserved {
    fn default() -> Self {
        Self::new()
    }
}
