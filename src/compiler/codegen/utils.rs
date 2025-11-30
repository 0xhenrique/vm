// Utility functions for the compiler

use super::Compiler;

impl Compiler {
    // Check if a name is a builtin function
    pub(super) fn is_builtin_function(name: &str) -> bool {
        matches!(name,
            // Arithmetic
            "+" | "-" | "*" | "/" | "%" | "neg" |
            // Comparison
            "<=" | "<" | ">" | ">=" | "==" | "!=" |
            // List operations
            "cons" | "car" | "cdr" | "list?" | "append" | "list-ref" | "list-length" | "null?" | "list" |
            // Type predicates
            "integer?" | "boolean?" | "function?" | "closure?" | "procedure?" | "number?" |
            // String operations
            "string?" | "symbol?" | "symbol->string" | "string->symbol" |
            "string-length" | "substring" | "string-append" | "string->list" |
            "list->string" | "char-code" | "number->string" | "string->number" |
            "string-split" | "string-join" | "string-trim" | "string-replace" |
            "string-starts-with?" | "string-ends-with?" | "string-contains?" |
            "string-upcase" | "string-downcase" |
            // File I/O
            "read-file" | "write-file" | "file-exists?" | "write-binary-file" | "load" | "require" |
            // HashMap operations
            "hashmap?" | "hashmap-get" | "hashmap-set" | "hashmap-keys" |
            "hashmap-values" | "hashmap-contains-key?" | "hash-map" |
            // Vector operations
            "vector?" | "vector-ref" | "vector-set" | "vector-push" | "vector-pop" |
            "vector-length" | "vector" |
            // Type conversions
            "list->vector" | "vector->list" |
            // Metaprogramming & Reflection
            "eval" |
            "function-arity" | "function-params" | "closure-captured" | "function-name" |
            // Other
            "get-args" | "print"
        )
    }

    /// Generate a helpful suggestion for an undefined variable name
    /// Uses Levenshtein distance to find similar names
    pub(super) fn suggest_similar_name(&self, undefined_name: &str) -> String {
        let mut all_names = Vec::new();

        // Collect all possible names
        all_names.extend(self.local_bindings.keys().cloned());
        all_names.extend(self.pattern_bindings.keys().cloned());
        all_names.extend(self.param_names.iter().cloned());
        all_names.extend(self.global_vars.keys().cloned());
        all_names.extend(self.known_globals.iter().cloned());
        all_names.extend(self.functions.keys().cloned());
        all_names.extend(self.known_functions.iter().cloned());

        // Find the most similar name using Levenshtein distance
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for name in &all_names {
            let distance = Self::levenshtein_distance(undefined_name, name);
            // Only consider names within edit distance of 3
            if distance < best_distance && distance <= 3 {
                best_distance = distance;
                best_match = Some(name.clone());
            }
        }

        if let Some(similar_name) = best_match {
            format!("Did you mean '{}'? Check your spelling or define the variable before using it.", similar_name)
        } else {
            "Make sure the variable is defined before using it. Use 'defvar' to define global variables, or check if it's in scope.".to_string()
        }
    }

    /// Calculate Levenshtein distance between two strings
    /// This measures the minimum number of single-character edits needed to transform one string into another
    pub(super) fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for (i, &c1) in s1_chars.iter().enumerate() {
            for (j, &c2) in s2_chars.iter().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,     // deletion
                        matrix[i + 1][j] + 1      // insertion
                    ),
                    matrix[i][j] + cost           // substitution
                );
            }
        }

        matrix[len1][len2]
    }
}
