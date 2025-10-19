use crate::ast::*;
use crate::errors::CompileError;
use crate::BuildTarget; // Import the BuildTarget enum
use crate::token::TokenKind;
use crate::vdom::VNode;
use std::collections::HashMap;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType, EntityType,
};

/// A symbol table to track function indices.
struct FuncSymbolTable {
    funcs: HashMap<String, u32>,
}
impl FuncSymbolTable { fn new() -> Self { Self { funcs: HashMap::new() } } }


/// The code generator, responsible for emitting Wasm bytecode.
pub struct CodeGenerator {
    
    func_symbols: FuncSymbolTable,
    // Per-function state
    local_symbol_table: HashMap<String, u32>,
    local_count: u32,
    target: BuildTarget,
}

impl CodeGenerator {
    /// Creates a new CodeGenerator for a specific build target.
    pub fn new(target: BuildTarget) -> Self {
        Self {
            //module: Module::new(),
            func_symbols: FuncSymbolTable::new(),
            local_symbol_table: HashMap::new(),
            local_count: 0,
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
        self.local_count = 0;

        // Register function parameters as locals (they start at index 0)
        for param in &func.parameters {
            self.local_symbol_table.insert(param.name.value.clone(), self.local_count);
            self.local_count += 1;
        }

        let local_types: Vec<ValType> = func.body.statements.iter().filter_map(|s| {
            if let Statement::Let(_) = s { Some(ValType::I32) } else { None }
        }).collect();
        let mut f = Function::new_with_locals_types(local_types);

        for stmt in &func.body.statements {
            self.generate_statement(stmt, &mut f)?;
        }

        f.instruction(&Instruction::I32Const(0));
        f.instruction(&Instruction::End);
        Ok(f)
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
                let local_index = self.local_count;
                self.local_symbol_table.insert(let_stmt.name.value.clone(), local_index);
                self.local_count += 1;
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
            Expression::StringLiteral(_s) => {
                // For now, strings are represented as i32 (pointer to string data)
                // In a full implementation, we'd allocate string in WASM memory
                // For now, push a dummy value
                f.instruction(&Instruction::I32Const(0));
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

    fn get_import_index(&self, name: &str) -> Result<u32, CompileError> {
        self.func_symbols.funcs.get(name).copied().ok_or_else(|| {
            CompileError::Generic(format!("Import '{}' not found", name))
        })
    }
}