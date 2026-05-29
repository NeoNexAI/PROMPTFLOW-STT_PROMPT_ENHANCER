/// SQLite storage via rusqlite
/// Table: usage_log (id, timestamp, mode, provider, tokens, cost_usd, input_len, output_len)
pub struct Database;

impl Database {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
