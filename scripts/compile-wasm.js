#!/usr/bin/env node

/**
 * Compile WAT (WebAssembly Text) to WASM binary
 * Simple compiler using wabt npm package
 */

const fs = require('fs');
const path = require('path');

// For now, create a simple WASM module manually
// In the future, this would use the full RavensOne compiler

function createSimpleWasmModule() {
    // This is a hand-coded WASM binary for a simple counter
    // Magic number + version
    const wasmModule = new Uint8Array([
        0x00, 0x61, 0x73, 0x6d, // \0asm magic number
        0x01, 0x00, 0x00, 0x00, // version 1

        // Type section
        0x01, // section code
        0x07, // section size
        0x01, // number of types
        0x60, // function type
        0x00, // no parameters
        0x01, 0x7f, // one i32 result

        // Function section
        0x03, // section code
        0x02, // section size
        0x01, // number of functions
        0x00, // function 0 uses type 0

        // Memory section
        0x05, // section code
        0x03, // section size
        0x01, // number of memories
        0x00, 0x01, // min 1 page

        // Export section
        0x07, // section code
        0x0e, // section size
        0x02, // number of exports

        // Export memory
        0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, // "memory"
        0x02, 0x00, // memory 0

        // Export add function
        0x03, 0x61, 0x64, 0x64, // "add"
        0x00, 0x00, // function 0

        // Code section
        0x0a, // section code
        0x09, // section size
        0x01, // number of functions
        0x07, // function body size
        0x00, // no locals
        0x41, 0x05, // i32.const 5
        0x41, 0x03, // i32.const 3
        0x6a, // i32.add
        0x0b, // end
    ]);

    return wasmModule;
}

// Create output directory
const distDir = path.join(__dirname, '..', 'dist');
if (!fs.existsSync(distDir)) {
    fs.mkdirSync(distDir, { recursive: true });
}

// Generate WASM file
const wasmBytes = createSimpleWasmModule();
const outputPath = path.join(distDir, 'counter.wasm');

fs.writeFileSync(outputPath, wasmBytes);

console.log('✅ Compiled WASM module');
console.log(`📦 Output: ${outputPath}`);
console.log(`📏 Size: ${wasmBytes.length} bytes`);
