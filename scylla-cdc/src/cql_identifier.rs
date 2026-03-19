/// A CQL identifier (keyspace name, table name, column name, etc.) that is
/// always properly quoted when formatted for use in CQL statements.
///
/// The inner value stores the name exactly as provided.
/// The [`Display`](std::fmt::Display) implementation wraps it in double quotes
/// with embedded double-quote characters escaped by doubling them
/// (`"` -> `""`), following the CQL grammar.
///
/// Names coming from the Scylla driver's cluster metadata are already in
/// their canonical internal form, so they should be passed through as-is.
/// The quoting ensures CQL does not case-fold or reject reserved words.
#[derive(Clone, Debug)]
pub struct CqlIdentifier {
    raw: String,
    quoted: String,
}

impl CqlIdentifier {
    /// Creates a new `CqlIdentifier`, preserving the name exactly as given.
    ///
    /// The name will be double-quoted when formatted, preventing CQL from case-folding it.
    /// Pass names in their canonical form (e.g. as read from cluster metadata).
    pub fn new(name: impl Into<String>) -> Self {
        let raw = name.into();
        let quoted = format!(r#""{}""#, raw.replace('"', r#""""#));
        CqlIdentifier { raw, quoted }
    }

    /// Returns the raw (unquoted) identifier name.
    pub fn as_raw(&self) -> &str {
        &self.raw
    }
}

impl PartialEq for CqlIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl Eq for CqlIdentifier {}

impl std::hash::Hash for CqlIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl std::fmt::Display for CqlIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.quoted)
    }
}

impl From<&str> for CqlIdentifier {
    fn from(s: &str) -> Self {
        CqlIdentifier::new(s)
    }
}

impl From<String> for CqlIdentifier {
    fn from(s: String) -> Self {
        CqlIdentifier::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::CqlIdentifier;

    #[test]
    fn quotes_lowercase_name() {
        let id = CqlIdentifier::new("my_table");
        assert_eq!(id.to_string(), r#""my_table""#);
    }

    #[test]
    fn preserves_mixed_case() {
        let id = CqlIdentifier::new("MyKeyspace");
        assert_eq!(id.to_string(), r#""MyKeyspace""#);
    }

    #[test]
    fn quotes_uppercase_name() {
        let id = CqlIdentifier::new("ALL_CAPS");
        assert_eq!(id.to_string(), r#""ALL_CAPS""#);
    }

    #[test]
    fn as_raw_returns_original_name() {
        let id = CqlIdentifier::new("My_Keyspace");
        assert_eq!(id.as_raw(), "My_Keyspace");
    }

    #[test]
    fn from_str_and_from_string_are_equivalent() {
        let from_str: CqlIdentifier = "SomeName".into();
        let from_string: CqlIdentifier = String::from("SomeName").into();
        assert_eq!(from_str, from_string);
    }

    #[test]
    fn equal_names_are_equal() {
        let a = CqlIdentifier::new("ks");
        let b = CqlIdentifier::new("ks");
        assert_eq!(a, b);
    }

    #[test]
    fn different_names_are_not_equal() {
        let a = CqlIdentifier::new("ks");
        let b = CqlIdentifier::new("KS");
        assert_ne!(a, b);
    }

    #[test]
    fn clone_is_equal_to_original() {
        let id = CqlIdentifier::new("tbl");
        assert_eq!(id, id.clone());
    }

    #[test]
    fn usable_as_hash_map_key() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(CqlIdentifier::new("ks"), 1);
        assert_eq!(map[&CqlIdentifier::new("ks")], 1);
    }

    #[test]
    fn escapes_single_embedded_double_quote() {
        let id = CqlIdentifier::new(r#"has"quote"#);
        assert_eq!(id.to_string(), r#""has""quote""#);
    }

    #[test]
    fn escapes_multiple_embedded_double_quotes() {
        let id = CqlIdentifier::new(r#"a"b"c"#);
        assert_eq!(id.to_string(), r#""a""b""c""#);
    }

    #[test]
    fn as_raw_preserves_embedded_quotes() {
        let id = CqlIdentifier::new(r#"has"quote"#);
        assert_eq!(id.as_raw(), r#"has"quote"#);
    }

    #[test]
    fn empty_name_produces_empty_quoted_identifier() {
        let id = CqlIdentifier::new("");
        assert_eq!(id.to_string(), r#""""#);
        assert_eq!(id.as_raw(), "");
    }

    #[test]
    fn whitespace_name_is_preserved() {
        let id = CqlIdentifier::new("  spaces  ");
        assert_eq!(id.to_string(), r#""  spaces  ""#);
        assert_eq!(id.as_raw(), "  spaces  ");
    }

    #[test]
    fn special_characters_are_preserved() {
        let id = CqlIdentifier::new("col$name.with-special!chars");
        assert_eq!(id.to_string(), r#""col$name.with-special!chars""#);
        assert_eq!(id.as_raw(), "col$name.with-special!chars");
    }

    #[test]
    fn display_composes_into_dotted_keyspace_table() {
        let ks = CqlIdentifier::new("MyKeyspace");
        let tbl = CqlIdentifier::new("my_table");
        assert_eq!(format!("{ks}.{tbl}"), r#""MyKeyspace"."my_table""#);
    }
}
