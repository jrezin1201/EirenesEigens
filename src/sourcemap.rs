use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source map for mapping WASM bytecode back to RavensOne source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    /// Version of the source map format (always 3)
    pub version: u32,

    /// Original source file name
    pub file: String,

    /// List of source files
    pub sources: Vec<String>,

    /// List of source contents (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources_content: Option<Vec<String>>,

    /// Source map mappings in VLQ format
    pub mappings: String,

    /// List of symbol names
    pub names: Vec<String>,
}

/// A single mapping entry
#[derive(Debug, Clone)]
pub struct Mapping {
    /// Generated column in WASM
    pub generated_column: u32,

    /// Source file index
    pub source_index: u32,

    /// Original line in source
    pub original_line: u32,

    /// Original column in source
    pub original_column: u32,

    /// Symbol name index (optional)
    pub name_index: Option<u32>,
}

/// Source map builder
pub struct SourceMapBuilder {
    file: String,
    sources: Vec<String>,
    sources_content: Vec<String>,
    names: Vec<String>,
    name_map: HashMap<String, u32>,
    mappings: Vec<Vec<Mapping>>,
}

impl SourceMapBuilder {
    pub fn new(file: String) -> Self {
        Self {
            file,
            sources: Vec::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            name_map: HashMap::new(),
            mappings: Vec::new(),
        }
    }

    /// Add a source file
    pub fn add_source(&mut self, source: String, content: String) -> u32 {
        let index = self.sources.len() as u32;
        self.sources.push(source);
        self.sources_content.push(content);
        index
    }

    /// Add a symbol name
    pub fn add_name(&mut self, name: String) -> u32 {
        if let Some(&index) = self.name_map.get(&name) {
            return index;
        }

        let index = self.names.len() as u32;
        self.name_map.insert(name.clone(), index);
        self.names.push(name);
        index
    }

    /// Add a mapping
    pub fn add_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        source_index: u32,
        original_line: u32,
        original_column: u32,
        name: Option<String>,
    ) {
        // Ensure we have enough lines
        while self.mappings.len() <= generated_line as usize {
            self.mappings.push(Vec::new());
        }

        let name_index = name.map(|n| self.add_name(n));

        let mapping = Mapping {
            generated_column,
            source_index,
            original_line,
            original_column,
            name_index,
        };

        self.mappings[generated_line as usize].push(mapping);
    }

    /// Build the final source map
    pub fn build(self) -> SourceMap {
        let mappings = encode_mappings(&self.mappings);

        SourceMap {
            version: 3,
            file: self.file,
            sources: self.sources,
            sources_content: Some(self.sources_content),
            mappings,
            names: self.names,
        }
    }
}

/// Encode mappings to VLQ format
fn encode_mappings(mappings: &[Vec<Mapping>]) -> String {
    let mut result = String::new();
    let mut prev_source = 0i32;
    let mut prev_original_line = 0i32;
    let mut prev_original_column = 0i32;
    let mut prev_name = 0i32;

    for (line_idx, line_mappings) in mappings.iter().enumerate() {
        if line_idx > 0 {
            result.push(';');
        }

        let mut prev_generated_column = 0i32;

        for (seg_idx, mapping) in line_mappings.iter().enumerate() {
            if seg_idx > 0 {
                result.push(',');
            }

            // Generated column
            encode_vlq(mapping.generated_column as i32 - prev_generated_column, &mut result);
            prev_generated_column = mapping.generated_column as i32;

            // Source file index
            encode_vlq(mapping.source_index as i32 - prev_source, &mut result);
            prev_source = mapping.source_index as i32;

            // Original line
            encode_vlq(mapping.original_line as i32 - prev_original_line, &mut result);
            prev_original_line = mapping.original_line as i32;

            // Original column
            encode_vlq(mapping.original_column as i32 - prev_original_column, &mut result);
            prev_original_column = mapping.original_column as i32;

            // Name index (optional)
            if let Some(name_idx) = mapping.name_index {
                encode_vlq(name_idx as i32 - prev_name, &mut result);
                prev_name = name_idx as i32;
            }
        }
    }

    result
}

/// Encode a single value using Variable Length Quantity (VLQ)
fn encode_vlq(value: i32, output: &mut String) {
    const VLQ_BASE: i32 = 32;
    const VLQ_BASE_MASK: i32 = VLQ_BASE - 1;
    const VLQ_CONTINUATION_BIT: i32 = VLQ_BASE;

    let mut value = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };

    loop {
        let mut digit = value & VLQ_BASE_MASK;
        value >>= 5;

        if value > 0 {
            digit |= VLQ_CONTINUATION_BIT;
        }

        output.push(encode_base64_digit(digit as u8));

        if value == 0 {
            break;
        }
    }
}

/// Encode a base64 digit
fn encode_base64_digit(digit: u8) -> char {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    BASE64_CHARS[digit as usize] as char
}

/// Generate inline source map comment for WASM
pub fn generate_inline_sourcemap_comment(sourcemap: &SourceMap) -> String {
    let json = serde_json::to_string(sourcemap).unwrap();
    let encoded = base64::encode(&json);
    format!("//# sourceMappingURL=data:application/json;charset=utf-8;base64,{}", encoded)
}

/// Stack trace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Parse a WASM stack trace and map it to source locations
pub fn map_stack_trace(
    wasm_trace: &str,
    sourcemap: &SourceMap,
) -> Vec<StackFrame> {
    let mut frames = Vec::new();

    for line in wasm_trace.lines() {
        if let Some(frame) = parse_stack_frame(line, sourcemap) {
            frames.push(frame);
        }
    }

    frames
}

/// Decode VLQ-encoded mappings string into structured mappings
fn decode_mappings(mappings: &str) -> Option<Vec<Vec<Mapping>>> {
    let mut result = Vec::new();
    let mut prev_source = 0i32;
    let mut prev_original_line = 0i32;
    let mut prev_original_column = 0i32;
    let mut prev_name = 0i32;

    for line in mappings.split(';') {
        let mut line_mappings = Vec::new();
        let mut prev_generated_column = 0i32;

        for segment in line.split(',') {
            if segment.is_empty() {
                continue;
            }

            let mut values = Vec::new();
            let mut chars = segment.chars().peekable();

            // Decode VLQ values
            while chars.peek().is_some() {
                if let Some(value) = decode_vlq_value(&mut chars) {
                    values.push(value);
                } else {
                    return None;
                }
            }

            // A segment must have at least 4 values (generated_column, source_index, original_line, original_column)
            if values.len() < 4 {
                continue;
            }

            // Decode relative values to absolute
            let generated_column = prev_generated_column + values[0];
            let source_index = prev_source + values[1];
            let original_line = prev_original_line + values[2];
            let original_column = prev_original_column + values[3];
            let name_index = if values.len() >= 5 {
                prev_name += values[4];
                Some(prev_name as u32)
            } else {
                None
            };

            prev_generated_column = generated_column;
            prev_source = source_index;
            prev_original_line = original_line;
            prev_original_column = original_column;

            line_mappings.push(Mapping {
                generated_column: generated_column as u32,
                source_index: source_index as u32,
                original_line: original_line as u32,
                original_column: original_column as u32,
                name_index,
            });
        }

        result.push(line_mappings);
    }

    Some(result)
}

/// Decode a single VLQ value from a character iterator
fn decode_vlq_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<i32> {
    const VLQ_BASE: i32 = 32;
    const VLQ_CONTINUATION_BIT: i32 = VLQ_BASE;

    let mut result = 0;
    let mut shift = 0;

    loop {
        let ch = chars.next()?;
        let digit = decode_base64_char(ch)? as i32;

        let continuation = (digit & VLQ_CONTINUATION_BIT) != 0;
        let value = digit & (VLQ_BASE - 1);

        result += value << shift;
        shift += 5;

        if !continuation {
            break;
        }
    }

    // Convert from variable-length quantity to signed integer
    let is_negative = (result & 1) != 0;
    result >>= 1;

    Some(if is_negative { -result } else { result })
}

/// Decode a base64 character to its numeric value
fn decode_base64_char(ch: char) -> Option<u8> {
    match ch {
        'A'..='Z' => Some((ch as u8) - b'A'),
        'a'..='z' => Some((ch as u8) - b'a' + 26),
        '0'..='9' => Some((ch as u8) - b'0' + 52),
        '+' => Some(62),
        '/' => Some(63),
        _ => None,
    }
}

/// Look up the mapping for a specific generated location
fn lookup_mapping(mappings: &[Vec<Mapping>], line: u32, column: u32) -> Option<&Mapping> {
    // Get the mappings for this line
    let line_mappings = mappings.get(line as usize)?;

    // Find the mapping that covers this column
    // Mappings are sorted by generated_column, so we find the last one <= our column
    let mut best_mapping: Option<&Mapping> = None;

    for mapping in line_mappings {
        if mapping.generated_column <= column {
            best_mapping = Some(mapping);
        } else {
            break;
        }
    }

    best_mapping
}

fn parse_stack_frame(line: &str, sourcemap: &SourceMap) -> Option<StackFrame> {
    // Simple parser for stack trace lines like:
    // "at functionName (file.wasm:line:column)"

    let parts: Vec<&str> = line.split('(').collect();
    if parts.len() < 2 {
        return None;
    }

    let function_name = parts[0].trim().trim_start_matches("at ").to_string();
    let location = parts[1].trim_end_matches(')');

    let location_parts: Vec<&str> = location.split(':').collect();
    if location_parts.len() < 3 {
        return None;
    }

    let _file = location_parts[0];
    let wasm_line: u32 = location_parts[1].parse().ok()?;
    let wasm_column: u32 = location_parts[2].parse().ok()?;

    // Decode the source map and find the mapping for this location
    let decoded_mappings = decode_mappings(&sourcemap.mappings)?;
    let mapping = lookup_mapping(&decoded_mappings, wasm_line, wasm_column)?;

    // Get the source file name
    let source_file = sourcemap.sources.get(mapping.source_index as usize)?.clone();

    Some(StackFrame {
        function_name,
        file: source_file,
        line: mapping.original_line,
        column: mapping.original_column,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sourcemap_builder() {
        let mut builder = SourceMapBuilder::new("output.wasm".to_string());

        let source_idx = builder.add_source("main.raven".to_string(), "component App() {}".to_string());

        builder.add_mapping(0, 0, source_idx, 0, 0, Some("App".to_string()));
        builder.add_mapping(0, 10, source_idx, 0, 10, None);

        let sourcemap = builder.build();

        assert_eq!(sourcemap.version, 3);
        assert_eq!(sourcemap.file, "output.wasm");
        assert_eq!(sourcemap.sources.len(), 1);
        assert!(!sourcemap.mappings.is_empty());
    }

    #[test]
    fn test_vlq_encoding() {
        let mut output = String::new();
        encode_vlq(0, &mut output);
        assert_eq!(output, "A");

        output.clear();
        encode_vlq(1, &mut output);
        assert_eq!(output, "C");

        output.clear();
        encode_vlq(-1, &mut output);
        assert_eq!(output, "D");
    }

    #[test]
    fn test_vlq_decode() {
        // Test decoding "A" -> 0
        let mut chars = "A".chars().peekable();
        let value = decode_vlq_value(&mut chars);
        assert_eq!(value, Some(0));

        // Test decoding "C" -> 1
        let mut chars = "C".chars().peekable();
        let value = decode_vlq_value(&mut chars);
        assert_eq!(value, Some(1));

        // Test decoding "D" -> -1
        let mut chars = "D".chars().peekable();
        let value = decode_vlq_value(&mut chars);
        assert_eq!(value, Some(-1));
    }

    #[test]
    fn test_mapping_decode_and_lookup() {
        // Create a source map with known mappings
        let mut builder = SourceMapBuilder::new("output.wasm".to_string());
        let source_idx = builder.add_source("main.raven".to_string(), "fn main() {}".to_string());

        // Add some mappings
        // Line 0, column 0 maps to source line 0, column 0
        builder.add_mapping(0, 0, source_idx, 0, 0, Some("main".to_string()));
        // Line 0, column 10 maps to source line 0, column 3
        builder.add_mapping(0, 10, source_idx, 0, 3, None);

        let sourcemap = builder.build();

        // Test that we can decode the mappings
        let decoded = decode_mappings(&sourcemap.mappings).expect("Failed to decode mappings");
        assert_eq!(decoded.len(), 1); // One line of mappings
        assert_eq!(decoded[0].len(), 2); // Two segments on that line

        // Test lookup
        let mapping = lookup_mapping(&decoded, 0, 0).expect("Failed to find mapping");
        assert_eq!(mapping.generated_column, 0);
        assert_eq!(mapping.original_line, 0);
        assert_eq!(mapping.original_column, 0);

        let mapping = lookup_mapping(&decoded, 0, 15).expect("Failed to find mapping");
        assert_eq!(mapping.generated_column, 10);
        assert_eq!(mapping.original_line, 0);
        assert_eq!(mapping.original_column, 3);
    }

    #[test]
    fn test_stack_trace_mapping() {
        // Create a source map
        let mut builder = SourceMapBuilder::new("output.wasm".to_string());
        let source_idx = builder.add_source("test.raven".to_string(), "fn foo() {}".to_string());
        builder.add_mapping(5, 10, source_idx, 3, 4, Some("foo".to_string()));
        let sourcemap = builder.build();

        // Test parsing a stack frame
        let stack_line = "at myFunction (output.wasm:5:10)";
        let frames = map_stack_trace(stack_line, &sourcemap);

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].function_name, "myFunction");
        assert_eq!(frames[0].file, "test.raven");
        assert_eq!(frames[0].line, 3);
        assert_eq!(frames[0].column, 4);
    }
}
