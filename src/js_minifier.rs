// JS Minifier - Simple JavaScript minification for production builds
//
// Provides basic minification:
// - Removes comments
// - Removes unnecessary whitespace
// - Removes empty lines
// - Compresses consecutive spaces

pub struct JSMinifier;

impl JSMinifier {
    pub fn new() -> Self {
        JSMinifier
    }

    /// Minifies JavaScript code
    pub fn minify(&self, code: &str) -> String {
        let mut result = String::with_capacity(code.len());
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_comment = false;
        let mut in_line_comment = false;
        let mut last_char = ' ';
        let mut escape_next = false;

        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            let next_char = if i + 1 < chars.len() { chars[i + 1] } else { '\0' };

            // Handle escape sequences in strings
            if escape_next {
                result.push(c);
                escape_next = false;
                last_char = c;
                i += 1;
                continue;
            }

            if in_string {
                result.push(c);
                if c == '\\' {
                    escape_next = true;
                } else if c == string_char {
                    in_string = false;
                }
                last_char = c;
                i += 1;
                continue;
            }

            // Start of multi-line comment
            if !in_line_comment && c == '/' && next_char == '*' {
                in_comment = true;
                i += 2;
                continue;
            }

            // End of multi-line comment
            if in_comment && c == '*' && next_char == '/' {
                in_comment = false;
                i += 2;
                last_char = ' ';
                continue;
            }

            // Skip content inside multi-line comments
            if in_comment {
                i += 1;
                continue;
            }

            // Start of line comment
            if c == '/' && next_char == '/' {
                in_line_comment = true;
                i += 2;
                continue;
            }

            // End of line comment
            if in_line_comment && (c == '\n' || c == '\r') {
                in_line_comment = false;
                // Preserve line break if it's semantically important
                if !last_char.is_whitespace() {
                    result.push(' ');
                    last_char = ' ';
                }
                i += 1;
                continue;
            }

            // Skip content inside line comments
            if in_line_comment {
                i += 1;
                continue;
            }

            // Start of string
            if c == '"' || c == '\'' || c == '`' {
                in_string = true;
                string_char = c;
                result.push(c);
                last_char = c;
                i += 1;
                continue;
            }

            // Handle whitespace
            if c.is_whitespace() {
                // Only add space if needed (between identifiers/keywords)
                if !last_char.is_whitespace()
                    && self.needs_space_before(last_char)
                    && self.needs_space_after(next_char) {
                    result.push(' ');
                    last_char = ' ';
                }
                i += 1;
                continue;
            }

            // Add character
            result.push(c);
            last_char = c;
            i += 1;
        }

        result
    }

    /// Checks if space is needed before this character
    fn needs_space_before(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '$'
    }

    /// Checks if space is needed after this character
    fn needs_space_after(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '$'
    }

    /// Minifies with options
    pub fn minify_with_options(&self, code: &str, preserve_license: bool) -> String {
        if preserve_license {
            // Extract license comments (/*! ... */)
            let mut result = String::new();
            let mut remaining = code;

            while let Some(start) = remaining.find("/*!") {
                if let Some(end) = remaining[start..].find("*/") {
                    let license = &remaining[start..start + end + 2];
                    result.push_str(license);
                    result.push('\n');
                    remaining = &remaining[start + end + 2..];
                } else {
                    break;
                }
            }

            result.push_str(&self.minify(code));
            result
        } else {
            self.minify(code)
        }
    }

    /// Returns statistics about minification
    pub fn stats(&self, original: &str, minified: &str) -> MinifyStats {
        MinifyStats {
            original_size: original.len(),
            minified_size: minified.len(),
            reduction_bytes: original.len() - minified.len(),
            reduction_percent: if original.len() > 0 {
                ((original.len() - minified.len()) as f64 / original.len() as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug)]
pub struct MinifyStats {
    pub original_size: usize,
    pub minified_size: usize,
    pub reduction_bytes: usize,
    pub reduction_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_comments() {
        let minifier = JSMinifier::new();
        let code = r#"
// This is a comment
function hello() {
    /* This is a
       multi-line comment */
    return "world";
}
"#;
        let minified = minifier.minify(code);
        assert!(!minified.contains("//"));
        assert!(!minified.contains("/*"));
        assert!(minified.contains("function hello"));
        assert!(minified.contains("return"));
    }

    #[test]
    fn test_minify_whitespace() {
        let minifier = JSMinifier::new();
        let code = "function   hello  (  )   {   return   'world'  ;  }";
        let minified = minifier.minify(code);
        assert!(minified.len() < code.len());
        assert!(minified.contains("function hello"));
    }

    #[test]
    fn test_preserve_strings() {
        let minifier = JSMinifier::new();
        let code = r#"let msg = "Hello  World  with  spaces";"#;
        let minified = minifier.minify(code);
        assert!(minified.contains("Hello  World  with  spaces"));
    }

    #[test]
    fn test_preserve_template_literals() {
        let minifier = JSMinifier::new();
        let code = "let msg = `Hello ${name}`;";
        let minified = minifier.minify(code);
        assert!(minified.contains("`Hello ${name}`"));
    }

    #[test]
    fn test_stats() {
        let minifier = JSMinifier::new();
        let code = "function hello() {\n    return 'world';\n}";
        let minified = minifier.minify(code);
        let stats = minifier.stats(code, &minified);

        assert_eq!(stats.original_size, code.len());
        assert_eq!(stats.minified_size, minified.len());
        assert!(stats.reduction_bytes > 0);
        assert!(stats.reduction_percent > 0.0);
    }
}
