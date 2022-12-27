/// Format the given `query` such that it can be used in a `LIKE` expression for SQLite.
pub fn to_sqlite_search(query: &str) -> String {
    format!("%{query}%")
}
