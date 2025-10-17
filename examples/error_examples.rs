// Example demonstrating RavensOne's beautiful error messages
// Run with: cargo run --example error_examples

use ravensone_compiler::diagnostics::*;

fn main() {
    println!("🎨 RavensOne Enhanced Diagnostics Demo\n");
    println!("=========================================\n");

    // Example source code with various errors
    let source_code = r#"let x = 10;
let y = "hello";
let z = x + y;  // Type error here
let unused = 42;
return x;"#;

    let mut collector = DiagnosticCollector::new();

    // Example 1: Type mismatch error
    println!("📍 Example 1: Type Mismatch Error\n");
    let type_error = DiagnosticBuilder::type_mismatch(
        "Int",
        "String",
        SourceLocation {
            file: "example.raven".to_string(),
            line: 3,
            column: 9,
            length: 5,
        },
    );
    collector.add(type_error.clone());
    println!("{}", type_error.display(Some(source_code)));

    // Example 2: Undefined variable
    println!("\n📍 Example 2: Undefined Variable\n");
    let undef_var = DiagnosticBuilder::undefined_variable(
        "unknown_var",
        SourceLocation {
            file: "example.raven".to_string(),
            line: 5,
            column: 8,
            length: 11,
        },
    );
    println!("{}", undef_var.display(None));

    // Example 3: Did you mean...? suggestion
    println!("\n📍 Example 3: 'Did You Mean?' Suggestion\n");
    let similar = find_similar("Signa", &["Signal", "Computed", "Effect", "Resource"]);
    let func_error = DiagnosticBuilder::undefined_function(
        "Signa",
        SourceLocation {
            file: "example.raven".to_string(),
            line: 1,
            column: 10,
            length: 5,
        },
        similar.as_deref(),
    );
    println!("{}", func_error.display(None));

    // Example 4: Syntax error
    println!("\n📍 Example 4: Syntax Error\n");
    let syntax_error = DiagnosticBuilder::syntax_error(
        "`;`",
        "`}`",
        SourceLocation {
            file: "example.raven".to_string(),
            line: 10,
            column: 15,
            length: 1,
        },
    );
    println!("{}", syntax_error.display(None));

    // Example 5: Warning - unused variable
    println!("\n📍 Example 5: Unused Variable Warning\n");
    let unused_warning = DiagnosticBuilder::unused_variable(
        "unused",
        SourceLocation {
            file: "example.raven".to_string(),
            line: 4,
            column: 5,
            length: 6,
        },
    );
    collector.add(unused_warning.clone());
    println!("{}", unused_warning.display(Some(source_code)));

    // Example 6: Borrow checker error
    println!("\n📍 Example 6: Borrow Checker Error\n");
    let borrow_error = DiagnosticBuilder::borrow_error(
        "cannot borrow `x` as mutable because it is also borrowed as immutable",
        SourceLocation {
            file: "example.raven".to_string(),
            line: 7,
            column: 10,
            length: 1,
        },
    );
    println!("{}", borrow_error.display(None));

    // Example 7: JSX error
    println!("\n📍 Example 7: JSX Error\n");
    let jsx_error = DiagnosticBuilder::jsx_error(
        "unclosed JSX tag: expected `</div>`, found end of file",
        SourceLocation {
            file: "component.raven".to_string(),
            line: 15,
            column: 1,
            length: 0,
        },
    );
    println!("{}", jsx_error.display(None));

    // Example 8: Custom error with multiple suggestions
    println!("\n📍 Example 8: Custom Error with Multiple Suggestions\n");
    let custom_error = Diagnostic::error("component name must start with an uppercase letter")
        .at(SourceLocation {
            file: "example.raven".to_string(),
            line: 20,
            column: 11,
            length: 7,
        })
        .with_code("E007")
        .with_suggestion("rename to `MyComponent`")
        .with_suggestion("use PascalCase for component names")
        .with_note("components are distinguished from HTML elements by capitalization");
    println!("{}", custom_error.display(None));

    // Show summary with collector
    println!("\n=========================================");
    println!("📊 Diagnostic Summary\n");
    println!("{}", collector.display_all(Some(source_code)));

    println!("\n✨ These beautiful error messages help developers:");
    println!("   • Quickly locate errors with line/column numbers");
    println!("   • See source context with syntax highlighting");
    println!("   • Get helpful suggestions for fixes");
    println!("   • Understand the root cause with notes");
    println!("   • Track multiple issues at once");
}
