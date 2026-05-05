//! AST-based symbol extraction via tree-sitter.
//!
//! Behind the `tree-sitter` feature flag (opt-in). When enabled, extracts
//! symbols using exact AST parsing instead of regex. Languages currently
//! supported: Rust, Go, Python, TypeScript/JavaScript, Bash.
//!
//! Default lean build does NOT include tree-sitter - regex extraction in
//! `symbols.rs` is the fallback.

#[cfg(feature = "tree-sitter")]
mod ts_impl {
    use crate::symbols::{Symbol, SymbolKind};
    use anyhow::{Context, Result};
    use std::path::Path;
    use streaming_iterator::StreamingIterator;
    use tree_sitter::{Language, Parser, Query, QueryCursor};

    /// Extract symbols from source using tree-sitter for the given language id.
    ///
    /// Returns Ok(empty) if the language is not supported by AST extraction.
    pub fn extract_symbols_ast(path: &Path, source: &str, lang: &str) -> Result<Vec<Symbol>> {
        let language = match lang_to_tree_sitter(lang) {
            Some(l) => l,
            None => return Ok(Vec::new()),
        };

        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .context("set tree-sitter language")?;

        let tree = parser
            .parse(source, None)
            .context("tree-sitter parse failed")?;

        let query_text = query_for_language(lang).context("no query for language")?;
        let query = Query::new(&language, query_text)?;
        let mut cursor = QueryCursor::new();

        let path_str = path.to_string_lossy().into_owned();
        let mut symbols = Vec::new();
        let bytes = source.as_bytes();

        let mut matches = cursor.matches(&query, tree.root_node(), bytes);
        while let Some(m) = matches.next() {
            for capture in m.captures {
                let capture_name = &query.capture_names()[capture.index as usize];
                let node = capture.node;
                let name_text = node.utf8_text(bytes).unwrap_or("").to_string();
                if name_text.is_empty() {
                    continue;
                }

                let mut kind = capture_kind(capture_name);
                // Promote function -> method when inside a class/struct body.
                if matches!(kind, SymbolKind::Function) && is_inside_class_or_struct(&node) {
                    kind = SymbolKind::Method;
                }
                let line = (node.start_position().row + 1) as u32;
                let parent_node = node.parent().unwrap_or(node);
                let signature = parent_node.utf8_text(bytes).unwrap_or("").to_string();
                let signature = first_line(&signature).to_string();

                symbols.push(Symbol {
                    name: name_text,
                    kind,
                    language: lang.to_string(),
                    path: path_str.clone(),
                    line,
                    signature,
                    doc: None,
                });
            }
        }

        Ok(symbols)
    }

    fn first_line(s: &str) -> &str {
        s.lines().next().unwrap_or(s)
    }

    fn is_inside_class_or_struct(node: &tree_sitter::Node) -> bool {
        let mut current = node.parent();
        while let Some(n) = current {
            let kind = n.kind();
            if matches!(
                kind,
                "class_definition"
                    | "class_declaration"
                    | "struct_item"
                    | "impl_item"
                    | "interface_declaration"
            ) {
                return true;
            }
            current = n.parent();
        }
        false
    }

    fn capture_kind(name: &str) -> SymbolKind {
        if name.contains("type")
            || name.contains("struct")
            || name.contains("enum")
            || name.contains("trait")
            || name.contains("interface")
        {
            SymbolKind::Type
        } else if name.contains("class") {
            SymbolKind::Class
        } else if name.contains("method") {
            SymbolKind::Method
        } else if name.contains("constant") || name.contains("const") {
            SymbolKind::Constant
        } else {
            SymbolKind::Function
        }
    }

    fn lang_to_tree_sitter(lang: &str) -> Option<Language> {
        match lang {
            "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "go" => Some(tree_sitter_go::LANGUAGE.into()),
            "python" => Some(tree_sitter_python::LANGUAGE.into()),
            "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            "javascript" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            "bash" => Some(tree_sitter_bash::LANGUAGE.into()),
            _ => None,
        }
    }

    fn query_for_language(lang: &str) -> Option<&'static str> {
        match lang {
            "rust" => Some(RUST_QUERY),
            "go" => Some(GO_QUERY),
            "python" => Some(PYTHON_QUERY),
            "typescript" | "javascript" => Some(TS_QUERY),
            "bash" => Some(BASH_QUERY),
            _ => None,
        }
    }

    const RUST_QUERY: &str = r#"
    (function_item name: (identifier) @function)
    (struct_item name: (type_identifier) @struct)
    (enum_item name: (type_identifier) @enum)
    (trait_item name: (type_identifier) @trait)
    (const_item name: (identifier) @constant)
    (static_item name: (identifier) @constant)
    "#;

    const GO_QUERY: &str = r#"
    (function_declaration name: (identifier) @function)
    (method_declaration name: (field_identifier) @method)
    (type_declaration (type_spec name: (type_identifier) @type))
    (const_declaration (const_spec name: (identifier) @constant))
    "#;

    const PYTHON_QUERY: &str = r#"
    (function_definition name: (identifier) @function)
    (class_definition name: (identifier) @class)
    "#;

    const TS_QUERY: &str = r#"
    (function_declaration name: (identifier) @function)
    (class_declaration name: (type_identifier) @class)
    (interface_declaration name: (type_identifier) @interface)
    (type_alias_declaration name: (type_identifier) @type)
    "#;

    const BASH_QUERY: &str = r#"
    (function_definition name: (word) @function)
    "#;
}

#[cfg(feature = "tree-sitter")]
pub use ts_impl::extract_symbols_ast;

/// Stub when feature is disabled. Always returns empty.
#[cfg(not(feature = "tree-sitter"))]
pub fn extract_symbols_ast(
    _path: &std::path::Path,
    _source: &str,
    _lang: &str,
) -> anyhow::Result<Vec<crate::symbols::Symbol>> {
    Ok(Vec::new())
}

/// Whether AST extraction is available in this build.
pub const AST_ENABLED: bool = cfg!(feature = "tree-sitter");

#[cfg(all(test, feature = "tree-sitter"))]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn extracts_rust_function_via_ast() {
        let src = "pub fn validate_email(s: &str) -> bool { true }\n";
        let syms = extract_symbols_ast(&PathBuf::from("foo.rs"), src, "rust").unwrap();
        assert!(!syms.is_empty());
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"validate_email"));
    }

    #[test]
    fn extracts_go_function_and_type() {
        let src = "package main\n\ntype User struct { Name string }\n\nfunc GetUser() *User { return nil }\n";
        let syms = extract_symbols_ast(&PathBuf::from("foo.go"), src, "go").unwrap();
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"User"));
        assert!(names.contains(&"GetUser"));
    }

    #[test]
    fn extracts_python_class_method() {
        let src = "class UserService:\n    def get_user(self, id):\n        pass\n";
        let syms = extract_symbols_ast(&PathBuf::from("foo.py"), src, "python").unwrap();
        let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"UserService"));
        assert!(names.contains(&"get_user"));
    }

    #[test]
    fn unsupported_language_returns_empty() {
        let syms = extract_symbols_ast(&PathBuf::from("foo.cobol"), "x", "cobol").unwrap();
        assert!(syms.is_empty());
    }
}
