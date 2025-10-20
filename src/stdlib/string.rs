/// Standard library String type with enhanced operations
/// A heap-allocated, growable UTF-8 string type

pub const STRING_DEFINITION: &str = r#"
// String - A growable UTF-8 string
// Provides rich string manipulation capabilities
struct String {
    chars: Vec<u8>,  // UTF-8 bytes
    length: i32,     // Number of characters
}

impl String {
    // Create a new empty String
    fn new() -> String {
        return String {
            chars: Vec::new(),
            length: 0,
        };
    }

    // Create a String from a string literal
    fn from(s: &str) -> String {
        let chars = Vec::new();
        let len = 0;

        // Convert string to bytes
        for i in 0..s.len() {
            chars.push(s.as_bytes()[i]);
            len = len + 1;
        }

        return String {
            chars: chars,
            length: len,
        };
    }

    // Create a String with a specific capacity
    fn with_capacity(capacity: i32) -> String {
        return String {
            chars: Vec::with_capacity(capacity),
            length: 0,
        };
    }

    // Get the length of the string in characters
    fn len(self: &String) -> i32 {
        return self.length;
    }

    // Check if the string is empty
    fn is_empty(self: &String) -> bool {
        return self.length == 0;
    }

    // Convert to uppercase
    fn to_uppercase(self: &String) -> String {
        let result = String::new();

        for byte in &self.chars {
            // Simple ASCII uppercase conversion
            if byte >= 97 && byte <= 122 {  // a-z
                result.push_byte(byte - 32);
            } else {
                result.push_byte(byte);
            }
        }

        return result;
    }

    // Convert to lowercase
    fn to_lowercase(self: &String) -> String {
        let result = String::new();

        for byte in &self.chars {
            // Simple ASCII lowercase conversion
            if byte >= 65 && byte <= 90 {  // A-Z
                result.push_byte(byte + 32);
            } else {
                result.push_byte(byte);
            }
        }

        return result;
    }

    // Trim whitespace from both ends
    fn trim(self: &String) -> String {
        return self.trim_start().trim_end();
    }

    // Trim whitespace from the start
    fn trim_start(self: &String) -> String {
        let start = 0;

        // Find first non-whitespace character
        for i in 0..self.length {
            let byte = self.chars[i];
            if !self.is_whitespace(byte) {
                start = i;
                break;
            }
        }

        return self.substring(start, self.length);
    }

    // Trim whitespace from the end
    fn trim_end(self: &String) -> String {
        let end = self.length;

        // Find last non-whitespace character
        for i in (0..self.length).rev() {
            let byte = self.chars[i];
            if !self.is_whitespace(byte) {
                end = i + 1;
                break;
            }
        }

        return self.substring(0, end);
    }

    // Check if a character is whitespace
    fn is_whitespace(self: &String, byte: u8) -> bool {
        return byte == 32 || byte == 9 || byte == 10 || byte == 13;  // space, tab, newline, CR
    }

    // Split string by a delimiter
    fn split(self: &String, delimiter: &str) -> Vec<String> {
        let result = Vec::new();
        let current = String::new();
        let delim_bytes = delimiter.as_bytes();
        let delim_len = delimiter.len();

        for i in 0..self.length {
            // Check if we're at a delimiter
            let is_delim = true;
            if i + delim_len <= self.length {
                for j in 0..delim_len {
                    if self.chars[i + j] != delim_bytes[j] {
                        is_delim = false;
                        break;
                    }
                }
            } else {
                is_delim = false;
            }

            if is_delim {
                result.push(current);
                current = String::new();
                i = i + delim_len - 1;  // Skip delimiter
            } else {
                current.push_byte(self.chars[i]);
            }
        }

        // Add last part
        if current.len() > 0 {
            result.push(current);
        }

        return result;
    }

    // Split by lines (\\n or \\r\\n)
    fn lines(self: &String) -> Vec<String> {
        let result = Vec::new();
        let current = String::new();

        for i in 0..self.length {
            let byte = self.chars[i];

            if byte == 10 {  // \\n
                result.push(current);
                current = String::new();
            } else if byte == 13 && i + 1 < self.length && self.chars[i + 1] == 10 {  // \\r\\n
                result.push(current);
                current = String::new();
                i = i + 1;  // Skip \\n
            } else {
                current.push_byte(byte);
            }
        }

        // Add last line
        if current.len() > 0 {
            result.push(current);
        }

        return result;
    }

    // Check if string contains a substring
    fn contains(self: &String, needle: &str) -> bool {
        let needle_bytes = needle.as_bytes();
        let needle_len = needle.len();

        if needle_len > self.length {
            return false;
        }

        for i in 0..(self.length - needle_len + 1) {
            let matches = true;

            for j in 0..needle_len {
                if self.chars[i + j] != needle_bytes[j] {
                    matches = false;
                    break;
                }
            }

            if matches {
                return true;
            }
        }

        return false;
    }

    // Check if string starts with a prefix
    fn starts_with(self: &String, prefix: &str) -> bool {
        let prefix_bytes = prefix.as_bytes();
        let prefix_len = prefix.len();

        if prefix_len > self.length {
            return false;
        }

        for i in 0..prefix_len {
            if self.chars[i] != prefix_bytes[i] {
                return false;
            }
        }

        return true;
    }

    // Check if string ends with a suffix
    fn ends_with(self: &String, suffix: &str) -> bool {
        let suffix_bytes = suffix.as_bytes();
        let suffix_len = suffix.len();

        if suffix_len > self.length {
            return false;
        }

        let start = self.length - suffix_len;

        for i in 0..suffix_len {
            if self.chars[start + i] != suffix_bytes[i] {
                return false;
            }
        }

        return true;
    }

    // Find the first occurrence of a substring
    fn find(self: &String, needle: &str) -> Option<i32> {
        let needle_bytes = needle.as_bytes();
        let needle_len = needle.len();

        if needle_len > self.length {
            return Option::None;
        }

        for i in 0..(self.length - needle_len + 1) {
            let matches = true;

            for j in 0..needle_len {
                if self.chars[i + j] != needle_bytes[j] {
                    matches = false;
                    break;
                }
            }

            if matches {
                return Option::Some(i);
            }
        }

        return Option::None;
    }

    // Find the last occurrence of a substring
    fn rfind(self: &String, needle: &str) -> Option<i32> {
        let needle_bytes = needle.as_bytes();
        let needle_len = needle.len();

        if needle_len > self.length {
            return Option::None;
        }

        for i in (0..(self.length - needle_len + 1)).rev() {
            let matches = true;

            for j in 0..needle_len {
                if self.chars[i + j] != needle_bytes[j] {
                    matches = false;
                    break;
                }
            }

            if matches {
                return Option::Some(i);
            }
        }

        return Option::None;
    }

    // Replace all occurrences of a pattern
    fn replace(self: &String, from: &str, to: &str) -> String {
        let result = String::new();
        let from_bytes = from.as_bytes();
        let from_len = from.len();
        let to_bytes = to.as_bytes();
        let to_len = to.len();
        let i = 0;

        while i < self.length {
            // Check if we're at a match
            let matches = true;
            if i + from_len <= self.length {
                for j in 0..from_len {
                    if self.chars[i + j] != from_bytes[j] {
                        matches = false;
                        break;
                    }
                }
            } else {
                matches = false;
            }

            if matches {
                // Add replacement
                for j in 0..to_len {
                    result.push_byte(to_bytes[j]);
                }
                i = i + from_len;
            } else {
                // Copy character
                result.push_byte(self.chars[i]);
                i = i + 1;
            }
        }

        return result;
    }

    // Get a substring from start to end (exclusive)
    fn substring(self: &String, start: i32, end: i32) -> String {
        let result = String::new();

        for i in start..end {
            if i >= 0 && i < self.length {
                result.push_byte(self.chars[i]);
            }
        }

        return result;
    }

    // Get character at index
    fn char_at(self: &String, index: i32) -> Option<u8> {
        if index >= 0 && index < self.length {
            return Option::Some(self.chars[index]);
        }
        return Option::None;
    }

    // Append a string
    fn push_str(self: &mut String, s: &str) {
        let bytes = s.as_bytes();
        for i in 0..s.len() {
            self.push_byte(bytes[i]);
        }
    }

    // Append a single byte/character
    fn push_byte(self: &mut String, byte: u8) {
        self.chars.push(byte);
        self.length = self.length + 1;
    }

    // Remove and return the last character
    fn pop(self: &mut String) -> Option<u8> {
        if self.length == 0 {
            return Option::None;
        }

        let byte = self.chars.pop();
        self.length = self.length - 1;
        return byte;
    }

    // Clear the string
    fn clear(self: &mut String) {
        self.chars.clear();
        self.length = 0;
    }

    // Repeat the string n times
    fn repeat(self: &String, n: i32) -> String {
        let result = String::new();

        for i in 0..n {
            for byte in &self.chars {
                result.push_byte(byte);
            }
        }

        return result;
    }

    // Reverse the string
    fn reverse(self: &String) -> String {
        let result = String::new();

        for i in (0..self.length).rev() {
            result.push_byte(self.chars[i]);
        }

        return result;
    }

    // Join a slice of strings with a separator
    fn join(strings: &[String], separator: &str) -> String {
        let result = String::new();
        let sep_bytes = separator.as_bytes();

        for i in 0..strings.len() {
            // Add string
            for byte in &strings[i].chars {
                result.push_byte(byte);
            }

            // Add separator (except after last)
            if i < strings.len() - 1 {
                for j in 0..separator.len() {
                    result.push_byte(sep_bytes[j]);
                }
            }
        }

        return result;
    }

    // Pad string to a certain length with a character
    fn pad_start(self: &String, target_len: i32, pad_char: u8) -> String {
        if self.length >= target_len {
            return String::from(&self.to_str());
        }

        let result = String::new();
        let padding = target_len - self.length;

        for i in 0..padding {
            result.push_byte(pad_char);
        }

        for byte in &self.chars {
            result.push_byte(byte);
        }

        return result;
    }

    // Pad end of string
    fn pad_end(self: &String, target_len: i32, pad_char: u8) -> String {
        if self.length >= target_len {
            return String::from(&self.to_str());
        }

        let result = String::new();

        for byte in &self.chars {
            result.push_byte(byte);
        }

        let padding = target_len - self.length;
        for i in 0..padding {
            result.push_byte(pad_char);
        }

        return result;
    }

    // Convert to &str (placeholder for actual conversion)
    fn to_str(self: &String) -> &str {
        return "";  // Would return actual string slice
    }

    // Convert from bytes
    fn from_utf8(bytes: &[u8]) -> Result<String, String> {
        let s = String::new();

        for byte in bytes {
            s.push_byte(byte);
        }

        return Result::Ok(s);
    }

    // Get as bytes
    fn as_bytes(self: &String) -> &[u8] {
        return &self.chars[..];
    }

    // Count occurrences of a substring
    fn count(self: &String, needle: &str) -> i32 {
        let needle_bytes = needle.as_bytes();
        let needle_len = needle.len();
        let count = 0;

        if needle_len > self.length {
            return 0;
        }

        for i in 0..(self.length - needle_len + 1) {
            let matches = true;

            for j in 0..needle_len {
                if self.chars[i + j] != needle_bytes[j] {
                    matches = false;
                    break;
                }
            }

            if matches {
                count = count + 1;
            }
        }

        return count;
    }

    // Check if string is all alphabetic
    fn is_alphabetic(self: &String) -> bool {
        if self.length == 0 {
            return false;
        }

        for byte in &self.chars {
            let is_alpha = (byte >= 65 && byte <= 90) || (byte >= 97 && byte <= 122);
            if !is_alpha {
                return false;
            }
        }

        return true;
    }

    // Check if string is all numeric
    fn is_numeric(self: &String) -> bool {
        if self.length == 0 {
            return false;
        }

        for byte in &self.chars {
            if byte < 48 || byte > 57 {  // 0-9
                return false;
            }
        }

        return true;
    }

    // Check if string is all alphanumeric
    fn is_alphanumeric(self: &String) -> bool {
        if self.length == 0 {
            return false;
        }

        for byte in &self.chars {
            let is_alnum = (byte >= 48 && byte <= 57) ||  // 0-9
                          (byte >= 65 && byte <= 90) ||   // A-Z
                          (byte >= 97 && byte <= 122);    // a-z
            if !is_alnum {
                return false;
            }
        }

        return true;
    }
}

// Helper functions for common string operations

// Create string from integer
fn from_i32(n: i32) -> String {
    if n == 0 {
        return String::from("0");
    }

    let is_negative = n < 0;
    let mut num = if is_negative { -n } else { n };
    let result = String::new();

    // Build string in reverse
    while num > 0 {
        let digit = (num % 10) as u8 + 48;  // Convert to ASCII
        result.push_byte(digit);
        num = num / 10;
    }

    if is_negative {
        result.push_byte(45);  // '-'
    }

    return result.reverse();
}

// Create string from float
fn from_f64(f: f64) -> String {
    // Simplified implementation
    let int_part = f as i32;
    return from_i32(int_part);
}

// Parse integer from string
fn parse_i32(s: &str) -> Result<i32, String> {
    let bytes = s.as_bytes();
    let len = s.len();

    if len == 0 {
        return Result::Err("Empty string");
    }

    let is_negative = bytes[0] == 45;  // '-'
    let start = if is_negative { 1 } else { 0 };
    let result = 0;

    for i in start..len {
        let byte = bytes[i];

        if byte < 48 || byte > 57 {
            return Result::Err("Invalid digit");
        }

        let digit = (byte - 48) as i32;
        result = result * 10 + digit;
    }

    if is_negative {
        result = -result;
    }

    return Result::Ok(result);
}

// Concatenate two strings
fn concat(s1: &String, s2: &String) -> String {
    let result = String::new();

    for byte in &s1.chars {
        result.push_byte(byte);
    }

    for byte in &s2.chars {
        result.push_byte(byte);
    }

    return result;
}

// Format string (simplified implementation)
fn format(template: &str, args: &[String]) -> String {
    let result = String::from(template);

    // Replace {} placeholders with arguments
    for i in 0..args.len() {
        result = result.replace("{}", &args[i].to_str());
    }

    return result;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_definition_exists() {
        assert!(!STRING_DEFINITION.is_empty());
    }

    #[test]
    fn test_string_definition_contains_struct() {
        assert!(STRING_DEFINITION.contains("struct String"));
    }

    #[test]
    fn test_string_definition_contains_basic_methods() {
        assert!(STRING_DEFINITION.contains("fn new()"));
        assert!(STRING_DEFINITION.contains("fn from("));
        assert!(STRING_DEFINITION.contains("fn len("));
        assert!(STRING_DEFINITION.contains("fn is_empty("));
    }

    #[test]
    fn test_string_definition_contains_case_methods() {
        assert!(STRING_DEFINITION.contains("fn to_uppercase("));
        assert!(STRING_DEFINITION.contains("fn to_lowercase("));
    }

    #[test]
    fn test_string_definition_contains_trim_methods() {
        assert!(STRING_DEFINITION.contains("fn trim("));
        assert!(STRING_DEFINITION.contains("fn trim_start("));
        assert!(STRING_DEFINITION.contains("fn trim_end("));
    }

    #[test]
    fn test_string_definition_contains_search_methods() {
        assert!(STRING_DEFINITION.contains("fn contains("));
        assert!(STRING_DEFINITION.contains("fn starts_with("));
        assert!(STRING_DEFINITION.contains("fn ends_with("));
        assert!(STRING_DEFINITION.contains("fn find("));
        assert!(STRING_DEFINITION.contains("fn rfind("));
    }

    #[test]
    fn test_string_definition_contains_split_methods() {
        assert!(STRING_DEFINITION.contains("fn split("));
        assert!(STRING_DEFINITION.contains("fn lines("));
    }

    #[test]
    fn test_string_definition_contains_manipulation_methods() {
        assert!(STRING_DEFINITION.contains("fn replace("));
        assert!(STRING_DEFINITION.contains("fn substring("));
        assert!(STRING_DEFINITION.contains("fn repeat("));
        assert!(STRING_DEFINITION.contains("fn reverse("));
    }

    #[test]
    fn test_string_definition_contains_mutation_methods() {
        assert!(STRING_DEFINITION.contains("fn push_str("));
        assert!(STRING_DEFINITION.contains("fn push_byte("));
        assert!(STRING_DEFINITION.contains("fn pop("));
        assert!(STRING_DEFINITION.contains("fn clear("));
    }

    #[test]
    fn test_string_definition_contains_validation_methods() {
        assert!(STRING_DEFINITION.contains("fn is_alphabetic("));
        assert!(STRING_DEFINITION.contains("fn is_numeric("));
        assert!(STRING_DEFINITION.contains("fn is_alphanumeric("));
    }

    #[test]
    fn test_string_definition_contains_helpers() {
        assert!(STRING_DEFINITION.contains("fn from_i32"));
        assert!(STRING_DEFINITION.contains("fn parse_i32"));
        assert!(STRING_DEFINITION.contains("fn concat"));
        assert!(STRING_DEFINITION.contains("fn format"));
    }
}
