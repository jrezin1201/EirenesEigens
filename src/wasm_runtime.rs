// WebAssembly Runtime Support for RavensOne
// Memory management, string handling, and runtime imports

use wasm_encoder::*;

/// WASM memory configuration
pub const MEMORY_PAGE_SIZE: u32 = 65536; // 64KB per page
pub const INITIAL_PAGES: u32 = 1;
pub const MAX_PAGES: u32 = 100;

/// Memory layout
pub const HEAP_START: u32 = 1024; // First 1KB reserved for runtime
pub const STRING_TABLE_START: u32 = HEAP_START;

/// Runtime imports that RavensOne programs need
pub struct RuntimeImports {
    imports: Vec<(String, String, EntityType)>,
}

impl RuntimeImports {
    pub fn new() -> Self {
        let mut imports = Vec::new();

        // Console/debugging
        imports.push(("env".to_string(), "log".to_string(), EntityType::Function(0)));
        imports.push(("env".to_string(), "error".to_string(), EntityType::Function(0)));

        // DOM manipulation (for client-side)
        imports.push(("dom".to_string(), "createElement".to_string(), EntityType::Function(1)));
        imports.push(("dom".to_string(), "createTextNode".to_string(), EntityType::Function(2)));
        imports.push(("dom".to_string(), "setAttribute".to_string(), EntityType::Function(3)));
        imports.push(("dom".to_string(), "appendChild".to_string(), EntityType::Function(4)));
        imports.push(("dom".to_string(), "addEventListener".to_string(), EntityType::Function(5)));

        // Reactive runtime
        imports.push(("reactive".to_string(), "signal_new".to_string(), EntityType::Function(6)));
        imports.push(("reactive".to_string(), "signal_get".to_string(), EntityType::Function(7)));
        imports.push(("reactive".to_string(), "signal_set".to_string(), EntityType::Function(8)));
        imports.push(("reactive".to_string(), "signal_update".to_string(), EntityType::Function(9)));
        imports.push(("reactive".to_string(), "computed_new".to_string(), EntityType::Function(10)));
        imports.push(("reactive".to_string(), "effect_new".to_string(), EntityType::Function(11)));

        // HTTP/Fetch (for RPC)
        imports.push(("http".to_string(), "fetch".to_string(), EntityType::Function(12)));

        RuntimeImports { imports }
    }

    pub fn add_to_import_section(&self, section: &mut ImportSection, types: &TypeSection) {
        for (module, name, entity_type) in &self.imports {
            section.import(module, name, entity_type.clone());
        }
    }

    pub fn get_type_section(&self) -> TypeSection {
        let mut types = TypeSection::new();

        // Type 0: (i32, i32) -> void [log, error]
        types.function(vec![ValType::I32, ValType::I32], vec![]);

        // Type 1: (i32, i32) -> i32 [createElement]
        types.function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);

        // Type 2: (i32, i32) -> i32 [createTextNode]
        types.function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);

        // Type 3: (i32, i32, i32, i32, i32) -> void [setAttribute]
        types.function(vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], vec![]);

        // Type 4: (i32, i32) -> void [appendChild]
        types.function(vec![ValType::I32, ValType::I32], vec![]);

        // Type 5: (i32, i32, i32, i32) -> void [addEventListener]
        types.function(vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], vec![]);

        // Type 6: (i32) -> i32 [signal_new]
        types.function(vec![ValType::I32], vec![ValType::I32]);

        // Type 7: (i32) -> i32 [signal_get]
        types.function(vec![ValType::I32], vec![ValType::I32]);

        // Type 8: (i32, i32) -> void [signal_set]
        types.function(vec![ValType::I32, ValType::I32], vec![]);

        // Type 9: (i32, i32) -> void [signal_update]
        types.function(vec![ValType::I32, ValType::I32], vec![]);

        // Type 10: (i32) -> i32 [computed_new]
        types.function(vec![ValType::I32], vec![ValType::I32]);

        // Type 11: (i32) -> i32 [effect_new]
        types.function(vec![ValType::I32], vec![ValType::I32]);

        // Type 12: (i32, i32) -> i32 [fetch]
        types.function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);

        types
    }

    pub fn import_count(&self) -> u32 {
        self.imports.len() as u32
    }
}

impl Default for RuntimeImports {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory manager for WASM linear memory
pub struct MemoryManager {
    next_offset: u32,
    string_table: Vec<(u32, String)>, // (offset, string)
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            next_offset: STRING_TABLE_START,
            string_table: Vec::new(),
        }
    }

    /// Allocate a string in memory and return its offset
    pub fn allocate_string(&mut self, s: &str) -> u32 {
        let offset = self.next_offset;
        let bytes = s.as_bytes();
        self.next_offset += bytes.len() as u32 + 8; // length (4) + ptr (4) + data
        self.string_table.push((offset, s.to_string()));
        offset
    }

    /// Generate data section with pre-allocated strings
    pub fn create_data_section(&self) -> DataSection {
        let mut data = DataSection::new();

        for (offset, string) in &self.string_table {
            let bytes = string.as_bytes();
            // Store length (4 bytes) followed by string data
            let mut buffer = Vec::new();
            buffer.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buffer.extend_from_slice(bytes);

            data.active(
                0, // memory index
                &ConstExpr::i32_const(*offset as i32),
                buffer.into_iter(),
            );
        }

        data
    }

    /// Create memory section
    pub fn create_memory_section() -> MemorySection {
        let mut memory = MemorySection::new();
        memory.memory(MemoryType {
            minimum: INITIAL_PAGES as u64,
            maximum: Some(MAX_PAGES as u64),
            memory64: false,
            shared: false,
        });
        memory
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to generate common WASM instruction sequences
pub struct InstructionHelper;

impl InstructionHelper {
    /// Load string pointer and length onto stack
    pub fn load_string(f: &mut Function, offset: u32) {
        // Load length
        f.instruction(&Instruction::I32Const(offset as i32));
        f.instruction(&Instruction::I32Load(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }));

        // Load pointer (offset + 4)
        f.instruction(&Instruction::I32Const((offset + 4) as i32));
    }

    /// Call console.log with a string
    pub fn call_log(f: &mut Function, string_offset: u32, log_func_index: u32) {
        Self::load_string(f, string_offset);
        f.instruction(&Instruction::Call(log_func_index));
    }

    /// Allocate memory (bump allocator)
    pub fn malloc(f: &mut Function, size: u32, heap_ptr_global: u32) {
        // Get current heap pointer
        f.instruction(&Instruction::GlobalGet(heap_ptr_global));

        // Save current pointer (will be returned)
        f.instruction(&Instruction::LocalTee(0)); // Assuming local 0 is available

        // Increment heap pointer by size
        f.instruction(&Instruction::GlobalGet(heap_ptr_global));
        f.instruction(&Instruction::I32Const(size as i32));
        f.instruction(&Instruction::I32Add);
        f.instruction(&Instruction::GlobalSet(heap_ptr_global));
    }

    /// Store i32 value at memory address
    pub fn store_i32(f: &mut Function, addr: u32, value: i32) {
        f.instruction(&Instruction::I32Const(addr as i32));
        f.instruction(&Instruction::I32Const(value));
        f.instruction(&Instruction::I32Store(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }));
    }

    /// Load i32 value from memory address
    pub fn load_i32(f: &mut Function, addr: u32) {
        f.instruction(&Instruction::I32Const(addr as i32));
        f.instruction(&Instruction::I32Load(MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }));
    }
}

/// Table for indirect function calls (for event handlers, callbacks)
pub struct FunctionTable {
    functions: Vec<u32>,
}

impl FunctionTable {
    pub fn new() -> Self {
        FunctionTable {
            functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func_index: u32) -> u32 {
        let table_index = self.functions.len() as u32;
        self.functions.push(func_index);
        table_index
    }

    pub fn create_table_section(&self) -> TableSection {
        let mut table = TableSection::new();
        table.table(TableType {
            element_type: RefType::FUNCREF,
            minimum: self.functions.len() as u32,
            maximum: Some(self.functions.len() as u32),
        });
        table
    }

    pub fn create_element_section(&self) -> ElementSection {
        let mut elements = ElementSection::new();
        if !self.functions.is_empty() {
            elements.active(
                None,
                &ConstExpr::i32_const(0),
                Elements::Functions(&self.functions),
            );
        }
        elements
    }
}

impl Default for FunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Global variables for runtime state
pub struct GlobalsManager {
    globals: Vec<(ValType, bool)>, // (type, mutable)
}

impl GlobalsManager {
    pub fn new() -> Self {
        let mut globals = Vec::new();

        // Global 0: heap pointer (mutable i32)
        globals.push((ValType::I32, true));

        // Global 1: reactive context ID (mutable i32)
        globals.push((ValType::I32, true));

        GlobalsManager { globals }
    }

    pub fn create_global_section(&self) -> GlobalSection {
        let mut section = GlobalSection::new();

        for (i, (val_type, mutable)) in self.globals.iter().enumerate() {
            let init_value = match val_type {
                ValType::I32 => {
                    if i == 0 {
                        // Heap pointer starts at HEAP_START
                        ConstExpr::i32_const(HEAP_START as i32)
                    } else {
                        ConstExpr::i32_const(0)
                    }
                }
                ValType::I64 => ConstExpr::i64_const(0),
                ValType::F32 => ConstExpr::f32_const(0.0),
                ValType::F64 => ConstExpr::f64_const(0.0),
                ValType::V128 => panic!("V128 not supported"),
                ValType::Ref(_) => ConstExpr::ref_null(HeapType::Func),
            };

            section.global(
                GlobalType {
                    val_type: *val_type,
                    mutable: *mutable,
                },
                &init_value,
            );
        }

        section
    }

    pub fn heap_pointer_index(&self) -> u32 {
        0
    }

    pub fn reactive_context_index(&self) -> u32 {
        1
    }
}

impl Default for GlobalsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_imports() {
        let imports = RuntimeImports::new();
        assert!(imports.import_count() > 0);
        assert!(imports.import_count() >= 12); // At least our core imports
    }

    #[test]
    fn test_memory_manager() {
        let mut mem = MemoryManager::new();

        let offset1 = mem.allocate_string("hello");
        let offset2 = mem.allocate_string("world");

        assert!(offset2 > offset1);
        assert_eq!(mem.string_table.len(), 2);
    }

    #[test]
    fn test_function_table() {
        let mut table = FunctionTable::new();

        let idx1 = table.add_function(5);
        let idx2 = table.add_function(10);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(table.functions.len(), 2);
    }

    #[test]
    fn test_globals_manager() {
        let globals = GlobalsManager::new();

        assert_eq!(globals.heap_pointer_index(), 0);
        assert_eq!(globals.reactive_context_index(), 1);
        assert!(globals.globals.len() >= 2);
    }
}
