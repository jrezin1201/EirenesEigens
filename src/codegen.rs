use crate::ast::*;
use crate::errors::CompileError;
use crate::BuildTarget; // Import the BuildTarget enum
use crate::token::TokenKind;
use crate::vdom::VNode;
use crate::semantic_analyzer::ResolvedType;
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType, EntityType, MemoryType, MemorySection,
};

/// A symbol table to track function indices.
struct FuncSymbolTable {
    funcs: HashMap<String, u32>,
}
impl FuncSymbolTable { fn new() -> Self { Self { funcs: HashMap::new() } } }

/// Tracks struct field layouts for memory operations
#[derive(Debug, Clone)]
struct StructLayout {
    fields: Vec<(String, u32, ResolvedType)>,  // field_name, offset, type
    total_size: u32,
}

impl StructLayout {
    fn new() -> Self {
        Self {
            fields: Vec::new(),
            total_size: 0,
        }
    }

    fn add_field(&mut self, name: String, ty: ResolvedType) {
        let offset = self.total_size;
        let size = Self::type_size(&ty);
        self.fields.push((name, offset, ty));
        self.total_size += size;
    }

    fn get_field_offset(&self, field_name: &str) -> Option<u32> {
        self.fields
            .iter()
            .find(|(name, _, _)| name == field_name)
            .map(|(_, offset, _)| *offset)
    }

    fn type_size(ty: &ResolvedType) -> u32 {
        match ty {
            ResolvedType::Integer => 4,
            ResolvedType::Float => 8,
            ResolvedType::Bool => 4,
            ResolvedType::String => 4,  // Pointer to string data
            ResolvedType::Array(_) => 4,  // Pointer to array data
            ResolvedType::Struct(_) => 4,  // Pointer to struct data
            ResolvedType::Signal(_) => 4,  // Signal ID
            _ => 4,  // Default to pointer size
        }
    }
}

/// Tracks struct definitions and their layouts
struct StructTable {
    structs: HashMap<String, StructLayout>,
}

impl StructTable {
    fn new() -> Self {
        Self { structs: HashMap::new() }
    }

    fn define(&mut self, name: String, layout: StructLayout) {
        self.structs.insert(name, layout);
    }

    fn get_layout(&self, struct_name: &str) -> Option<&StructLayout> {
        self.structs.get(struct_name)
    }

    fn get_field_offset(&self, struct_name: &str, field_name: &str) -> Option<u32> {
        self.structs
            .get(struct_name)
            .and_then(|layout| layout.get_field_offset(field_name))
    }
}


/// The code generator, responsible for emitting Wasm bytecode.
pub struct CodeGenerator {

    func_symbols: FuncSymbolTable,
    struct_table: StructTable,
    // Per-function state
    local_symbol_table: HashMap<String, u32>,
    local_type_table: HashMap<String, String>,  // variable_name -> struct_type_name
    local_count: u32,
    heap_pointer: u32,  // Tracks the next available heap address
    target: BuildTarget,
}

impl CodeGenerator {
    /// Creates a new CodeGenerator for a specific build target.
    pub fn new(target: BuildTarget) -> Self {
        Self {
            //module: Module::new(),
            func_symbols: FuncSymbolTable::new(),
            struct_table: StructTable::new(),
            local_symbol_table: HashMap::new(),
            local_type_table: HashMap::new(),
            local_count: 0,
            heap_pointer: 0,  // Start heap at address 0
            target,
        }
    }

    /// The main entry point for generating a complete Wasm module from an AST.
    pub fn generate_program(&mut self, program: &Program) -> Result<Vec<u8>, CompileError> {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        let mut functions = FunctionSection::new();
        let mut imports = ImportSection::new();
        let mut exports = ExportSection::new();
        let mut code = CodeSection::new();
        let mut func_index_counter = 0;

        // --- Pass 0: Collect Struct Definitions ---
        // Build struct layouts from the AST
        for stmt in &program.statements {
            if let Statement::Struct(struct_def) = stmt {
                let mut layout = StructLayout::new();
                for (field_name, field_type) in &struct_def.fields {
                    // Convert TypeExpression to ResolvedType
                    let resolved_type = self.type_expression_to_resolved_type(field_type);
                    layout.add_field(field_name.value.clone(), resolved_type);
                }
                self.struct_table.define(struct_def.name.value.clone(), layout);
            }
        }

        // Add memory section (1 page = 64KB initially, can grow)
        let mut memory = MemorySection::new();
        memory.memory(MemoryType {
            minimum: 1,
            maximum: Some(10),
            memory64: false,
            shared: false,
        });
        module.section(&memory);

        // --- First Pass: Signatures and Imports ---
        // This pass collects all function signatures and builds the import table.
        for stmt in &program.statements {
            match stmt {
                Statement::ExternBlock(block) => {
                    for func in &block.functions {
                        let param_types: Vec<ValType> = func.parameters.iter().map(|_| ValType::I32).collect();
                        let type_index = types.len();
                        types.function(param_types, vec![]);
                        imports.import(&block.abi, &func.name.value, EntityType::Function(type_index));
                        self.func_symbols.funcs.insert(func.name.value.clone(), func_index_counter);
                        func_index_counter += 1;
                    }
                }
                Statement::Function(func_def) => {
                    // All functions, server or client, get a type signature.
                    let type_index = types.len();
                    // Generate parameter types (for now, all i32)
                    let param_types: Vec<ValType> = func_def.parameters.iter().map(|_| ValType::I32).collect();
                    types.function(param_types, vec![ValType::I32]);
                    functions.function(type_index);
                    self.func_symbols.funcs.insert(func_def.name.value.clone(), func_index_counter);

                    // Export the function if it's the main entry point or if we're on the server.
                    if func_def.name.value == "main" || (self.target == BuildTarget::Server && func_def.is_server) {
                        exports.export(&func_def.name.value, ExportKind::Func, func_index_counter);
                    }
                    func_index_counter += 1;
                }
                Statement::Component(comp) => {
                    // Export component as a function
                    let type_index = types.len();
                    types.function(vec![], vec![ValType::I32]);
                    functions.function(type_index);
                    self.func_symbols.funcs.insert(comp.name.value.clone(), func_index_counter);
                    exports.export(&comp.name.value, ExportKind::Func, func_index_counter);
                    func_index_counter += 1;
                }
                _ => {}
            }
        }

        // --- Second Pass: Code Generation ---
        // This pass generates the actual instruction bodies for the functions.
        for stmt in &program.statements {
            match stmt {
                Statement::Function(func_def) => {
                    match self.target {
                        BuildTarget::Client => {
                            if func_def.is_server {
                                // On the client, server functions get a stub.
                                code.function(&self.generate_rpc_stub(func_def)?);
                            } else {
                                // Normal functions get a full body.
                                code.function(&self.generate_function(func_def)?);
                            }
                        }
                        BuildTarget::Server => {
                            // On the server, we only compile server functions.
                            if func_def.is_server {
                                code.function(&self.generate_function(func_def)?);
                            }
                            // Non-server functions are ignored in a server build.
                        }
                    }
                }
                Statement::Component(comp) => {
                    // Generate component function
                    code.function(&self.generate_component(comp)?);
                }
                Statement::ImplBlock(_) => {
                    // Impl blocks don't generate code directly - methods are called through method call syntax
                    // For now, we skip impl blocks in codegen
                }
                Statement::Trait(_) => {
                    // Trait definitions don't generate code - they're just signatures
                    // Actual method implementations come from impl blocks
                }
                _ => {}
            }
        }

        module.section(&types);
        module.section(&imports);
        module.section(&functions);
        module.section(&exports);
        module.section(&code);

        Ok(module.finish())
    }

    /// Generates the full Wasm instruction body for a given function.
    fn generate_function(&mut self, func: &FunctionDefinition) -> Result<Function, CompileError> {
        self.local_symbol_table.clear();
        self.local_type_table.clear();
        self.local_count = 0;

        // Register function parameters as locals (they start at index 0)
        for param in &func.parameters {
            self.local_symbol_table.insert(param.name.value.clone(), self.local_count);
            self.local_count += 1;
        }

        // Count locals needed for the function body
        // This includes:
        // - let statements (1 local each)
        // - for-in loops (3 locals each: iterator, loop variable, option)
        // - match expressions (1 local for scrutinee)
        let local_count = self.count_required_locals(&func.body.statements);
        let local_types: Vec<ValType> = (0..local_count).map(|_| ValType::I32).collect();
        let mut f = Function::new_with_locals_types(local_types);

        for stmt in &func.body.statements {
            self.generate_statement(stmt, &mut f)?;
        }

        f.instruction(&Instruction::I32Const(0));
        f.instruction(&Instruction::End);
        Ok(f)
    }

    /// Counts the number of WASM locals needed for a list of statements
    fn count_required_locals(&self, stmts: &[Statement]) -> u32 {
        let mut count = 0;
        for stmt in stmts {
            count += self.count_statement_locals(stmt);
        }
        count
    }

    /// Counts locals needed for a single statement (recursively)
    fn count_statement_locals(&self, stmt: &Statement) -> u32 {
        match stmt {
            Statement::Let(_) => 1,
            Statement::ForIn(_) => {
                // For-in loops need 3 locals: iterator, loop variable, option
                3
            }
            Statement::If(if_stmt) => {
                let mut count = 0;
                count += self.count_required_locals(&if_stmt.then_branch.statements);
                if let Some(else_stmt) = &if_stmt.else_branch {
                    count += self.count_statement_locals(else_stmt);
                }
                count
            }
            Statement::While(while_stmt) => {
                self.count_required_locals(&while_stmt.body.statements)
            }
            Statement::For(for_stmt) => {
                let mut count = 0;
                if let Some(init) = &for_stmt.init {
                    count += self.count_statement_locals(init);
                }
                count += self.count_required_locals(&for_stmt.body.statements);
                if let Some(update) = &for_stmt.update {
                    count += self.count_statement_locals(update);
                }
                count
            }
            Statement::Expression(expr) => {
                // Match expressions allocate locals for scrutinee
                self.count_expression_locals(expr)
            }
            _ => 0,
        }
    }

    /// Counts locals needed for expressions (mainly for match)
    fn count_expression_locals(&self, expr: &Expression) -> u32 {
        match expr {
            Expression::Match(_) => {
                // Match expressions need 1 local for the scrutinee
                1
            }
            _ => 0,
        }
    }

    /// Generates a component as a WASM function
    fn generate_component(&mut self, comp: &ComponentDefinition) -> Result<Function, CompileError> {
        self.local_symbol_table.clear();
        self.local_count = 0;

        // Components just return their JSX body
        // For now, we'll generate a simple function that returns 0
        // In a full implementation, this would:
        // 1. Initialize reactive state (Signal<T> for each let binding)
        // 2. Set up effects for automatic re-rendering
        // 3. Generate and return the VDOM structure

        let mut f = Function::new(vec![]);

        // Generate the component body
        self.generate_expression(&comp.body, &mut f)?;

        f.instruction(&Instruction::End);
        Ok(f)
    }

    /// Generates a placeholder "teleporter pad" for a server function on the client.
    fn generate_rpc_stub(&mut self, _func: &FunctionDefinition) -> Result<Function, CompileError> {
        let mut f = Function::new(vec![]);
        // This generated function is a placeholder. A real implementation would:
        // 1. Serialize arguments into a buffer.
        // 2. Call a generic `_rpc_call` FFI function.
        // 3. Await and deserialize the result.
        
        // For now, it just returns a dummy value (e.g., -1 for an i32) to indicate it's a stub.
        f.instruction(&Instruction::I32Const(-1));
        f.instruction(&Instruction::End);
        Ok(f)
    }

    fn generate_statement(&mut self, stmt: &Statement, f: &mut Function) -> Result<(), CompileError> {
        match stmt {
            Statement::Let(let_stmt) => {
                self.generate_expression(&let_stmt.value, f)?;

                // Track the type if it's a struct literal
                if let Expression::StructLiteral(struct_lit) = &let_stmt.value {
                    self.local_type_table.insert(
                        let_stmt.name.value.clone(),
                        struct_lit.name.value.clone()
                    );
                }

                let local_index = self.local_count;
                self.local_symbol_table.insert(let_stmt.name.value.clone(), local_index);
                self.local_count += 1;
                f.instruction(&Instruction::LocalSet(local_index));
            }
            Statement::Assignment(assign_stmt) => {
                // Generate the value expression
                self.generate_expression(&assign_stmt.value, f)?;

                // Get the local index of the target variable
                let local_index = *self.local_symbol_table.get(&assign_stmt.target.value)
                    .ok_or_else(|| CompileError::Generic(format!(
                        "Codegen: undefined variable '{}' in assignment",
                        assign_stmt.target.value
                    )))?;

                // Set the local variable
                f.instruction(&Instruction::LocalSet(local_index));
            }
            Statement::Return(return_stmt) => {
                self.generate_expression(&return_stmt.value, f)?;
                f.instruction(&Instruction::End);
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr, f)?;
                // Expressions used as statements leave a value on the stack. Pop it.
                f.instruction(&Instruction::Drop);
            }
            Statement::If(if_stmt) => {
                self.generate_if_statement(if_stmt, f)?;
            }
            Statement::While(while_stmt) => {
                self.generate_while_statement(while_stmt, f)?;
            }
            Statement::For(for_stmt) => {
                self.generate_for_statement(for_stmt, f)?;
            }
            Statement::ForIn(for_in_stmt) => {
                self.generate_for_in_statement(for_in_stmt, f)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_if_statement(&mut self, stmt: &IfStatement, f: &mut Function) -> Result<(), CompileError> {
        // Generate condition
        self.generate_expression(&stmt.condition, f)?;

        // Start if block
        f.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));

        // Generate then branch
        for s in &stmt.then_branch.statements {
            self.generate_statement(s, f)?;
        }

        // Generate else branch if present
        if stmt.else_branch.is_some() {
            f.instruction(&Instruction::Else);
            if let Some(else_stmt) = &stmt.else_branch {
                self.generate_statement(else_stmt, f)?;
            }
        }

        // End if block
        f.instruction(&Instruction::End);
        Ok(())
    }

    fn generate_while_statement(&mut self, stmt: &WhileStatement, f: &mut Function) -> Result<(), CompileError> {
        // Start loop block
        f.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Generate condition
        self.generate_expression(&stmt.condition, f)?;

        // Invert condition (exit if false)
        f.instruction(&Instruction::I32Eqz);

        // Break out of loop if condition is false
        f.instruction(&Instruction::BrIf(1));

        // Generate loop body
        for s in &stmt.body.statements {
            self.generate_statement(s, f)?;
        }

        // Branch back to start of loop
        f.instruction(&Instruction::Br(0));

        // End loop block
        f.instruction(&Instruction::End);

        Ok(())
    }

    fn generate_for_statement(&mut self, stmt: &ForStatement, f: &mut Function) -> Result<(), CompileError> {
        // Generate init statement if present (runs once before loop)
        if let Some(init) = &stmt.init {
            self.generate_statement(init, f)?;
        }

        // Start loop block
        f.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Generate condition
        self.generate_expression(&stmt.condition, f)?;

        // Invert condition (exit if false)
        f.instruction(&Instruction::I32Eqz);

        // Break out of loop if condition is false
        f.instruction(&Instruction::BrIf(1));

        // Generate loop body
        for s in &stmt.body.statements {
            self.generate_statement(s, f)?;
        }

        // Generate update statement if present (runs after each iteration)
        if let Some(update) = &stmt.update {
            self.generate_statement(update, f)?;
        }

        // Branch back to start of loop
        f.instruction(&Instruction::Br(0));

        // End loop block
        f.instruction(&Instruction::End);

        Ok(())
    }

    fn generate_for_in_statement(&mut self, stmt: &ForInStatement, f: &mut Function) -> Result<(), CompileError> {
        // For-in loops use the Iterator protocol:
        // 1. Call into_iter() on the collection to get an iterator
        // 2. Loop: call next() on the iterator
        // 3. If Some(value), bind value to loop variable and execute body
        // 4. If None, exit loop
        //
        // This implementation generates WASM code that follows this protocol.
        // Note: This is a simplified version that works with the stdlib iterator types.
        // Full implementation would need:
        // - Dynamic dispatch for trait methods (into_iter, next)
        // - Option<T> enum discrimination in WASM
        // - Proper type inference from semantic analysis

        // Allocate a local for the iterator
        let iterator_local = self.local_count;
        self.local_count += 1;

        // Allocate a local for the loop variable
        let loop_var_local = self.local_count;
        self.local_symbol_table.insert(stmt.variable.value.clone(), loop_var_local);
        self.local_count += 1;

        // Allocate a local for the Option<T> result from next()
        let option_local = self.local_count;
        self.local_count += 1;

        // Step 1: Convert the collection into an iterator by calling into_iter()
        // For now, we'll assume the iterator expression evaluates to an iterable type
        // In a full implementation, we'd call the into_iter() method here
        self.generate_expression(&stmt.iterator, f)?;
        f.instruction(&Instruction::LocalSet(iterator_local));

        // Step 2: Generate the loop structure
        // We use a WASM loop block that:
        // - Calls next() on the iterator
        // - Checks if the result is Some or None
        // - If Some, extracts the value, binds it, executes body, continues loop
        // - If None, breaks out of the loop

        // Start the loop block
        f.instruction(&Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Step 3: Call next() on the iterator
        // For now, this is a simplified version that assumes:
        // - The iterator is stored in linear memory
        // - next() returns an Option<T> encoded as: tag (0=None, 1=Some) + value
        // In a full implementation, we'd:
        // 1. Load the iterator from local
        // 2. Call the next() method via dynamic dispatch
        // 3. Get the Option<T> result

        // Simplified: Load iterator (pointer to iterator object)
        f.instruction(&Instruction::LocalGet(iterator_local));

        // Call a hypothetical next() method that returns Option<T>
        // For this simplified version, we'll generate a placeholder that:
        // - Loads the option tag from memory (iterator_ptr + offset)
        // - If tag == 0 (None), exit loop
        // - If tag == 1 (Some), load value and continue

        // Load the option tag (first field of the Option<T> in memory)
        // Assume Option<T> layout: [tag: i32] [value: T]
        f.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,  // Tag is at offset 0
            align: 2,
            memory_index: 0,
        }));

        // Store the tag in option_local for checking
        f.instruction(&Instruction::LocalTee(option_local));

        // Check if tag == 0 (None)
        f.instruction(&Instruction::I32Eqz);

        // If tag is 0 (None), break out of the loop (exit to outer block)
        f.instruction(&Instruction::BrIf(1));

        // Step 4: If we're here, tag == 1 (Some), so extract the value
        // Load the value from Option<T> (second field after tag)
        f.instruction(&Instruction::LocalGet(iterator_local));
        f.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
            offset: 4,  // Value is at offset 4 (after the tag)
            align: 2,
            memory_index: 0,
        }));

        // Bind the value to the loop variable
        f.instruction(&Instruction::LocalSet(loop_var_local));

        // Step 5: Execute the loop body
        for s in &stmt.body.statements {
            self.generate_statement(s, f)?;
        }

        // Step 6: Continue the loop (branch back to the start)
        f.instruction(&Instruction::Br(0));

        // End the loop block
        f.instruction(&Instruction::End);

        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression, f: &mut Function) -> Result<(), CompileError> {
        match expr {
            Expression::IntegerLiteral(val) => {
                f.instruction(&Instruction::I32Const(*val as i32));
            }
            Expression::FloatLiteral(val) => {
                let float_val: f64 = val.parse().unwrap_or(0.0);
                f.instruction(&Instruction::F64Const(float_val));
            }
            Expression::BoolLiteral(val) => {
                f.instruction(&Instruction::I32Const(if *val { 1 } else { 0 }));
            }
            Expression::Identifier(ident) => {
                let local_index = self.local_symbol_table.get(&ident.value).ok_or_else(|| {
                    CompileError::Generic(format!("Codegen: undefined variable '{}'", ident.value))
                })?;
                f.instruction(&Instruction::LocalGet(*local_index));
            }
            Expression::Prefix(prefix) => {
                // Apply the prefix operator
                match &prefix.operator.kind {
                    TokenKind::Minus => {
                        // Negation: 0 - x  (push 0 first, then x, then subtract)
                        f.instruction(&Instruction::I32Const(0));
                        self.generate_expression(&prefix.right, f)?;
                        f.instruction(&Instruction::I32Sub);
                    }
                    TokenKind::Bang => {
                        // Logical NOT: x == 0
                        self.generate_expression(&prefix.right, f)?;
                        f.instruction(&Instruction::I32Eqz);
                    }
                    _ => return Err(CompileError::Generic(format!(
                        "Unsupported prefix operator: {:?}", prefix.operator.kind
                    ))),
                }
            }
            Expression::Infix(infix) => {
                self.generate_expression(&infix.left, f)?;
                self.generate_expression(&infix.right, f)?;

                match &infix.operator.kind {
                    TokenKind::Plus => { f.instruction(&Instruction::I32Add); }
                    TokenKind::Minus => { f.instruction(&Instruction::I32Sub); }
                    TokenKind::Star => { f.instruction(&Instruction::I32Mul); }
                    TokenKind::Eq => { f.instruction(&Instruction::I32Eq); }
                    TokenKind::NotEq => { f.instruction(&Instruction::I32Ne); }
                    TokenKind::LAngle => { f.instruction(&Instruction::I32LtS); }
                    TokenKind::RAngle => { f.instruction(&Instruction::I32GtS); }
                    TokenKind::LtEq => { f.instruction(&Instruction::I32LeS); }
                    TokenKind::GtEq => { f.instruction(&Instruction::I32GeS); }
                    _ => return Err(CompileError::Generic(format!(
                        "Unsupported operator: {:?}", infix.operator.kind
                    ))),
                }
            }
            Expression::Lambda(lambda) => {
                // For now, lambdas are compiled inline as code blocks
                // In a full implementation, they would be compiled to function table entries
                // For simple cases like `() => count + 1`, we just generate the body
                self.generate_expression(&lambda.body, f)?;
            }
            Expression::FunctionCall(call) => {
                // Generate function call
                self.generate_function_call(call, f)?;
            }
            Expression::JsxElement(jsx) => {
                // Generate JSX element as VDOM
                self.generate_jsx_element(jsx, f)?;
            }
            Expression::ArrayLiteral(array_lit) => {
                // Arrays in WASM linear memory layout:
                // [length: i32 (4 bytes)][element0: i32][element1: i32]...
                //
                // Steps:
                // 1. Calculate total size needed (4 bytes for length + 4 bytes per element)
                // 2. Allocate memory from heap
                // 3. Store length at offset 0
                // 4. Store each element at offset 4 + (index * element_size)
                // 5. Return pointer to the array

                let element_count = array_lit.elements.len();
                let element_size = 4; // Assume all elements are i32 for now
                let total_size = 4 + (element_count as u32 * element_size); // 4 bytes for length + elements

                // Allocate memory and get pointer
                let array_ptr = self.heap_pointer;
                self.heap_pointer += total_size;

                // Store the array length at offset 0
                f.instruction(&Instruction::I32Const(array_ptr as i32));
                f.instruction(&Instruction::I32Const(element_count as i32));
                f.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,  // 4-byte alignment for i32
                    memory_index: 0,
                }));

                // Store each element in the array
                for (index, element) in array_lit.elements.iter().enumerate() {
                    // Push the base pointer
                    f.instruction(&Instruction::I32Const(array_ptr as i32));

                    // Generate code for the element value
                    self.generate_expression(element, f)?;

                    // Store at offset: 4 (length field) + (index * element_size)
                    f.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                        offset: (4 + (index as u64 * element_size as u64)),
                        align: 2,  // 4-byte alignment for i32
                        memory_index: 0,
                    }));
                }

                // Push the array pointer as the result
                f.instruction(&Instruction::I32Const(array_ptr as i32));
            }
            Expression::StructLiteral(struct_lit) => {
                // Look up the struct layout
                let layout = self.struct_table.get_layout(&struct_lit.name.value)
                    .ok_or_else(|| CompileError::Generic(format!(
                        "Codegen: Unknown struct type '{}'",
                        struct_lit.name.value
                    )))?
                    .clone();

                // Allocate memory for the struct and push pointer onto stack
                let struct_ptr = self.heap_pointer;
                self.allocate_struct(layout.total_size, f);

                // For each field in the struct literal, store the value at the correct offset
                for (field_name, field_value) in &struct_lit.fields {
                    // Get the field offset from the layout
                    let offset = layout.get_field_offset(&field_name.value)
                        .ok_or_else(|| CompileError::Generic(format!(
                            "Codegen: Struct '{}' has no field '{}'",
                            struct_lit.name.value,
                            field_name.value
                        )))?;

                    // Push the base pointer + offset
                    f.instruction(&Instruction::I32Const(struct_ptr as i32));

                    // Generate code for the field value
                    self.generate_expression(field_value, f)?;

                    // Store the value at (base_ptr + offset)
                    // For now, assume all fields are i32 (we'll need to check type later)
                    f.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                        offset: offset as u64,
                        align: 2,  // 4-byte alignment for i32
                        memory_index: 0,
                    }));
                }

                // The struct pointer is already on the stack from allocate_struct
                // No need to push it again - we're reusing it as the return value
            }
            Expression::FieldAccess(field_access) => {
                // Generate code for the object expression to get the base pointer
                self.generate_expression(&field_access.object, f)?;

                // We need to determine the struct type to look up field offset
                // For now, we'll handle the case where the object is an identifier
                // In a full implementation, we'd track types through the semantic analyzer

                // Try to get the struct type from the object expression
                // This is a simplified approach - in production we'd use the type system
                let struct_name = self.infer_struct_type(&field_access.object)?;

                // Get the field offset
                let offset = self.struct_table.get_field_offset(&struct_name, &field_access.field.value)
                    .ok_or_else(|| CompileError::Generic(format!(
                        "Codegen: Struct '{}' has no field '{}'",
                        struct_name,
                        field_access.field.value
                    )))?;

                // Load the value from memory at (base_ptr + offset)
                // For now, assume all fields are i32
                f.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset: offset as u64,
                    align: 2,  // 4-byte alignment for i32
                    memory_index: 0,
                }));
            }
            Expression::Match(match_expr) => {
                // Generate code for the scrutinee and store it in a local
                self.generate_expression(&match_expr.scrutinee, f)?;

                // For simple match expressions with literal/wildcard patterns,
                // we generate nested if/else blocks
                // This implementation handles:
                // 1. Literal patterns (comparing values)
                // 2. Wildcard patterns (catch-all)
                // 3. Identifier patterns (binding values - treated as wildcard)

                if match_expr.arms.is_empty() {
                    // Empty match, push unit value
                    f.instruction(&Instruction::I32Const(0));
                } else {
                    // Generate nested if/else structure for pattern matching
                    self.generate_match_arms(&match_expr.arms, f)?;
                }
            }
            Expression::IndexAccess(index_expr) => {
                // Generate code for array indexing: arr[index]
                // In WASM, arrays are stored in linear memory
                // Array layout: [length (4 bytes)] [element0] [element1] ...

                // Generate the array expression (should produce a pointer)
                self.generate_expression(&index_expr.array, f)?;

                // Generate the index expression (should produce an i32)
                self.generate_expression(&index_expr.index, f)?;

                // Calculate the memory address: base_ptr + 4 + (index * element_size)
                // For now, assume all elements are 4 bytes (i32)
                // Multiply index by 4 (element size)
                f.instruction(&Instruction::I32Const(4));
                f.instruction(&Instruction::I32Mul);

                // Add offset for length field (skip first 4 bytes)
                f.instruction(&Instruction::I32Const(4));
                f.instruction(&Instruction::I32Add);

                // Add to base pointer
                f.instruction(&Instruction::I32Add);

                // Load the value from memory
                f.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,  // 4-byte alignment for i32
                    memory_index: 0,
                }));
            }
            Expression::TupleLiteral(tuple_lit) => {
                // For now, tuples are stored in linear memory similar to structs
                // Tuple layout: [element0] [element1] [element2] ...
                // Each element is 4 bytes (i32 for now)

                // Calculate tuple size (4 bytes per element)
                let tuple_size = (tuple_lit.elements.len() as u32) * 4;

                // Allocate memory for the tuple
                let tuple_ptr = self.heap_pointer;
                f.instruction(&Instruction::I32Const(tuple_ptr as i32));
                self.heap_pointer += tuple_size;

                // Store each element in memory
                for (i, elem) in tuple_lit.elements.iter().enumerate() {
                    // Push the base pointer
                    f.instruction(&Instruction::I32Const(tuple_ptr as i32));

                    // Generate the element value
                    self.generate_expression(elem, f)?;

                    // Store at offset (i * 4)
                    f.instruction(&Instruction::I32Store(wasm_encoder::MemArg {
                        offset: (i as u64) * 4,
                        align: 2,  // 4-byte alignment for i32
                        memory_index: 0,
                    }));
                }

                // Push the tuple pointer as the result
                f.instruction(&Instruction::I32Const(tuple_ptr as i32));
            }
            Expression::StringLiteral(_s) => {
                // For now, strings are represented as i32 (pointer to string data)
                // In a full implementation, we'd allocate string in WASM memory
                // For now, push a dummy value
                f.instruction(&Instruction::I32Const(0));
            }
            Expression::Borrow(_borrow_expr) => {
                // Borrowing in WASM is a no-op since everything is already a value or pointer
                // For now, just return a placeholder
                // In a full implementation with borrow checking, this would validate lifetime rules
                return Err(CompileError::Generic(
                    "Codegen: Borrow expressions not yet fully implemented in WASM codegen".to_string()
                ));
            }
            Expression::MutableBorrow(_borrow_expr) => {
                // Mutable borrowing in WASM is a no-op since everything is already a value or pointer
                // For now, just return a placeholder
                // In a full implementation with borrow checking, this would validate lifetime rules
                return Err(CompileError::Generic(
                    "Codegen: MutableBorrow expressions not yet fully implemented in WASM codegen".to_string()
                ));
            }
            Expression::Dereference(_deref_expr) => {
                // Dereferencing in WASM would load a value from a pointer
                // For now, return a placeholder
                // In a full implementation, this would load from memory
                return Err(CompileError::Generic(
                    "Codegen: Dereference expressions not yet fully implemented in WASM codegen".to_string()
                ));
            }
            Expression::Range(_range_expr) => {
                // Range expressions are placeholders for now
                // In a full implementation, we'd generate code for range creation
                // For now, push a dummy value
                f.instruction(&Instruction::I32Const(0));
            }
            Expression::TryOperator(_try_expr) => {
                // Try operator for error propagation
                // In a full implementation, this would:
                // 1. Evaluate the inner expression (which should return Result<T, E>)
                // 2. Check if it's Ok or Err
                // 3. If Ok, unwrap and continue
                // 4. If Err, propagate the error by returning early
                // For now, return a placeholder comment
                return Err(CompileError::Generic(
                    "// Try operator".to_string()
                ));
            }
        }
        Ok(())
    }

    fn generate_jsx_element(&mut self, jsx: &JsxElement, f: &mut Function) -> Result<(), CompileError> {
        // Convert JSX to VDOM structure and generate code to build it
        let vnode = self.jsx_to_vnode(jsx)?;

        // For now, we'll generate calls to DOM creation functions
        self.generate_vnode(&vnode, f)?;

        Ok(())
    }

    fn jsx_to_vnode(&self, jsx: &JsxElement) -> Result<VNode, CompileError> {
        let tag = jsx.opening_tag.name.value.clone();

        // Convert attributes
        let mut attrs = Vec::new();
        for attr in &jsx.opening_tag.attributes {
            // For now, we only handle string literal values
            // Event handlers and expressions need special handling
            let value = match &attr.value {
                Expression::StringLiteral(s) => s.clone(),
                Expression::Lambda(_) => {
                    // Event handler - we'll handle this specially
                    continue;
                }
                _ => "".to_string(), // Placeholder for other expressions
            };
            attrs.push((attr.name.value.clone(), value));
        }

        // Convert children
        let mut children = Vec::new();
        for child in &jsx.children {
            match child {
                JsxChild::Element(child_jsx) => {
                    children.push(self.jsx_to_vnode(child_jsx)?);
                }
                JsxChild::Text(text) => {
                    children.push(VNode::Text(text.clone()));
                }
                JsxChild::Expression(_expr) => {
                    // For now, skip expressions in children
                    // In full implementation, we'd evaluate and convert to text
                    children.push(VNode::Text("{{expr}}".to_string()));
                }
            }
        }

        Ok(VNode::Element { tag, attrs, children })
    }

    fn generate_vnode(&mut self, vnode: &VNode, f: &mut Function) -> Result<(), CompileError> {
        match vnode {
            VNode::Element { tag, attrs, children } => {
                // Call createElement(tag) -> elementId
                // For now, we'll just push a dummy element ID
                f.instruction(&Instruction::I32Const(0)); // dummy element ID

                // In a full implementation:
                // 1. Call createElement(tag_ptr, tag_len) to create element
                // 2. For each attribute, call setAttribute(elementId, name_ptr, name_len, value_ptr, value_len)
                // 3. For each child, recursively generate and call appendChild(parentId, childId)
                // 4. Return the element ID
            }
            VNode::Text(content) => {
                // Call createTextNode(content) -> nodeId
                f.instruction(&Instruction::I32Const(0)); // dummy node ID
            }
        }
        Ok(())
    }

    fn generate_function_call(&mut self, call: &FunctionCall, f: &mut Function) -> Result<(), CompileError> {
        // Check if this is a method call (function is a FieldAccess)
        if let Expression::FieldAccess(field_access) = &*call.function {
            return self.generate_method_call(field_access, &call.arguments, f);
        }

        // Generate arguments (push them onto the stack)
        for arg in &call.arguments {
            self.generate_expression(arg, f)?;
        }

        // Check if it's a built-in reactive function
        if let Expression::Identifier(ident) = &*call.function {
            match ident.value.as_str() {
                // Signal operations
                "signal_new" => {
                    // signal_new(initialValue) - returns signal ID
                    f.instruction(&Instruction::Call(self.get_import_index("signal_new")?));
                    return Ok(());
                }
                "signal_get" => {
                    // signal_get(signalId) - returns current value
                    f.instruction(&Instruction::Call(self.get_import_index("signal_get")?));
                    return Ok(());
                }
                "signal_set" => {
                    // signal_set(signalId, newValue)
                    f.instruction(&Instruction::Call(self.get_import_index("signal_set")?));
                    return Ok(());
                }
                "signal_update" => {
                    // signal_update(signalId, delta)
                    f.instruction(&Instruction::Call(self.get_import_index("signal_update")?));
                    return Ok(());
                }
                _ => {
                    // Look up user-defined function
                    if let Some(&func_idx) = self.func_symbols.funcs.get(&ident.value) {
                        f.instruction(&Instruction::Call(func_idx));
                    } else {
                        return Err(CompileError::Generic(format!(
                            "Undefined function: '{}'", ident.value
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_method_call(
        &mut self,
        field_access: &FieldAccessExpression,
        arguments: &[Expression],
        f: &mut Function,
    ) -> Result<(), CompileError> {
        let method_name = &field_access.field.value;

        // Handle array methods
        match method_name.as_str() {
            "len" => {
                // arr.len() - returns the length of the array
                // Array layout: [length (4 bytes)] [elements...]
                // Generate the array expression to get the pointer
                self.generate_expression(&field_access.object, f)?;

                // Load the length from memory at base_ptr + 0
                f.instruction(&Instruction::I32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 2,  // 4-byte alignment for i32
                    memory_index: 0,
                }));

                Ok(())
            }
            "push" => {
                // arr.push(value) - adds an element to the end of the array
                // This is complex because we need to:
                // 1. Read the current length
                // 2. Store the new value at base_ptr + 4 + (length * element_size)
                // 3. Increment the length
                // For now, return a placeholder error
                Err(CompileError::Generic(
                    "Codegen: arr.push() not yet fully implemented - requires mutable arrays".to_string()
                ))
            }
            "pop" => {
                // arr.pop() - removes and returns the last element
                // This would require:
                // 1. Read the current length
                // 2. Decrement the length
                // 3. Return the value at base_ptr + 4 + (new_length * element_size)
                // For now, return a placeholder error
                Err(CompileError::Generic(
                    "Codegen: arr.pop() not yet fully implemented - requires mutable arrays".to_string()
                ))
            }
            _ => {
                Err(CompileError::Generic(format!(
                    "Codegen: Unknown method '{}'", method_name
                )))
            }
        }
    }

    fn get_import_index(&self, name: &str) -> Result<u32, CompileError> {
        self.func_symbols.funcs.get(name).copied().ok_or_else(|| {
            CompileError::Generic(format!("Import '{}' not found", name))
        })
    }

    fn type_expression_to_resolved_type(&self, type_expr: &TypeExpression) -> ResolvedType {
        match type_expr {
            TypeExpression::Named(ident) => {
                match ident.value.as_str() {
                    "i32" => ResolvedType::Integer,
                    "f64" => ResolvedType::Float,
                    "bool" => ResolvedType::Bool,
                    "string" => ResolvedType::String,
                    _ => {
                        // Check if it's a struct type
                        if self.struct_table.structs.contains_key(&ident.value) {
                            ResolvedType::Struct(ident.value.clone())
                        } else {
                            ResolvedType::Unknown
                        }
                    }
                }
            }
            TypeExpression::Generic(ident, args) => {
                // Handle generic types like Array<T>
                if ident.value == "Array" && !args.is_empty() {
                    let inner_type = self.type_expression_to_resolved_type(&args[0]);
                    ResolvedType::Array(Box::new(inner_type))
                } else {
                    ResolvedType::Unknown
                }
            }
            TypeExpression::Tuple(types) => {
                // Convert tuple type expression to resolved tuple type
                let resolved_types: Vec<ResolvedType> = types.iter()
                    .map(|t| self.type_expression_to_resolved_type(t))
                    .collect();
                ResolvedType::Tuple(resolved_types)
            }
            TypeExpression::Reference(inner) => {
                // References are represented as pointers in WASM (i32)
                // For now, treat references the same as the underlying type
                // In a full implementation, we'd track reference semantics
                self.type_expression_to_resolved_type(inner)
            }
            TypeExpression::MutableReference(inner) => {
                // Mutable references are represented as pointers in WASM (i32)
                // For now, treat mutable references the same as the underlying type
                // In a full implementation, we'd track reference semantics
                self.type_expression_to_resolved_type(inner)
            }
            TypeExpression::Slice(inner) => {
                // Slices are array views - recursively process the inner type
                // Return ResolvedType::Array with the inner type
                let inner_type = self.type_expression_to_resolved_type(inner);
                ResolvedType::Array(Box::new(inner_type))
            }
        }
    }

    /// Allocates memory for a struct and returns the pointer
    fn allocate_struct(&mut self, size: u32, f: &mut Function) {
        // Push the current heap pointer (this will be the struct address)
        f.instruction(&Instruction::I32Const(self.heap_pointer as i32));

        // Update heap pointer for next allocation
        self.heap_pointer += size;
    }

    /// Attempts to infer the struct type from an expression
    /// This is a simplified version - in a full implementation, we'd use the semantic analyzer's type information
    fn infer_struct_type(&self, expr: &Expression) -> Result<String, CompileError> {
        match expr {
            Expression::Identifier(ident) => {
                // Look up the variable in our local type table
                if let Some(struct_name) = self.local_type_table.get(&ident.value) {
                    Ok(struct_name.clone())
                } else {
                    Err(CompileError::Generic(format!(
                        "Codegen: Cannot infer struct type for variable '{}' (not tracked)",
                        ident.value
                    )))
                }
            }
            Expression::StructLiteral(lit) => {
                // Easy case - we know the struct type from the literal
                Ok(lit.name.value.clone())
            }
            Expression::FieldAccess(field_access) => {
                // Recursively infer the type of nested field access
                // The type is determined by the base object
                self.infer_struct_type(&field_access.object)
            }
            _ => Err(CompileError::Generic(
                "Codegen: Cannot infer struct type for this expression".to_string()
            )),
        }
    }

    /// Generates WASM code for match arms using nested if/else blocks
    /// The scrutinee value is already on the stack when this is called
    fn generate_match_arms(&mut self, arms: &[MatchArm], f: &mut Function) -> Result<(), CompileError> {
        // We need to store the scrutinee in a local so we can compare it multiple times
        // Allocate a new local for the scrutinee value
        let scrutinee_local = self.local_count;
        self.local_count += 1;

        // Store the scrutinee value (which is already on the stack)
        f.instruction(&Instruction::LocalSet(scrutinee_local));

        // Generate nested if/else blocks for each arm
        self.generate_match_arm(arms, 0, scrutinee_local, f)?;

        Ok(())
    }

    /// Recursively generates WASM code for a single match arm
    /// This creates a nested if/else structure
    fn generate_match_arm(
        &mut self,
        arms: &[MatchArm],
        arm_index: usize,
        scrutinee_local: u32,
        f: &mut Function,
    ) -> Result<(), CompileError> {
        if arm_index >= arms.len() {
            // This should never happen due to exhaustiveness checking,
            // but if it does, return a dummy value
            f.instruction(&Instruction::I32Const(0));
            return Ok(());
        }

        let arm = &arms[arm_index];

        match &arm.pattern {
            Pattern::Wildcard | Pattern::Identifier(_) => {
                // Wildcard or identifier patterns always match
                // Just generate the body expression
                self.generate_expression(&arm.body, f)?;
            }
            Pattern::Literal(literal_expr) => {
                // For literal patterns, we need to compare the scrutinee with the literal

                // Load the scrutinee value
                f.instruction(&Instruction::LocalGet(scrutinee_local));

                // Generate the literal value
                self.generate_expression(literal_expr, f)?;

                // Compare them for equality
                f.instruction(&Instruction::I32Eq);

                // Start an if block (ValType::I32 means the block produces an i32 value)
                f.instruction(&Instruction::If(wasm_encoder::BlockType::Result(ValType::I32)));

                // If they match, generate this arm's body
                self.generate_expression(&arm.body, f)?;

                // Otherwise, try the next arm
                f.instruction(&Instruction::Else);

                // Recursively generate the next arm
                self.generate_match_arm(arms, arm_index + 1, scrutinee_local, f)?;

                // End the if block
                f.instruction(&Instruction::End);
            }
            Pattern::EnumVariant { name, fields } => {
                // For enum variants, we would need to:
                // 1. Load the tag field from the scrutinee (enum discriminant)
                // 2. Compare it with this variant's tag
                // 3. If they match, extract the fields if needed and generate the body
                //
                // For now, this is a simplified placeholder
                // In a full implementation, we'd:
                // - Load the enum tag: scrutinee_ptr + 0 (first field is always the tag)
                // - Compare it with the expected variant tag
                // - Extract variant data if fields are present

                // TODO: Implement full enum variant matching
                // For now, treat as a simple literal comparison with variant tag (0, 1, 2, etc.)
                return Err(CompileError::Generic(
                    "Codegen: Enum variant patterns not yet fully implemented in WASM codegen".to_string()
                ));
            }
        }

        Ok(())
    }
}