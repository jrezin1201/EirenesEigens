# Showcase Roadmap

| # | Name | Goal & Demo Value | RavensOne Features Exercised | Key Files/Artifacts | Acceptance Criteria | Stretch |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Hello Component | Prove end-to-end flow from `.raven` to generated HTML. | • CLI `init` → `build` • Component + text node • Dist output | `examples/hello` | `raven build` succeeds; `dist/index.html` renders "Hello" | Add attribute binding |
| 2 | Props & Slots | Demonstrate passing props between components. | • Component params • Nested components • `{expr}` slots | `examples/props-slots` | Props render correctly in HTML snapshot | Add default props |
| 3 | Conditional Rendering | Show simple `if`/`else` output. | • `if` statements • JSX branches | `examples/conditional` | Toggle flag in source toggles rendered block | Support ternary expressions |
| 4 | List Rendering | Render arrays with loops. | • `for` parsing • `<li>` generation | `examples/list` | Generated markup includes all list items | Add keyed diff hints |
| 5 | Styled Components | Highlight class/attribute support. | • Attribute literals • `{expr}` in attributes | `examples/styled` | CSS classes appear in generated HTML | Inline style expression |
| 6 | Static Site (Multi-page) | Showcase multiple entry components built together. | • Multiple `.raven` files • CLI build for each | `examples/static-site` | `raven build` outputs HTML for each page | Add simple nav links |
| 7 | Minimal Blog | Combine layout + posts data. | • Shared helpers • List rendering • Basic formatting | `examples/blog` | Markdown-like content renders using helpers | Add RSS export stub |
| 8 | Docs Site Template | Provide layout for documentation. | • Nested components • Sidebar + content | `examples/docs-site` | Build outputs navigation + content pages | Integrate search stub |
| 9 | Component Library Playground | Display interactive component catalogue (static). | • Reusable components • Props table | `examples/playground` | Each component documented on page | Hook up interactive toggles |
| 10 | Syntax Error Showcase | Demonstrate helpful error diagnostics. | • Lexer errors • Parser errors | `examples/errors` + logs | `raven build` fails with friendly error messages | Integrate docs links in errors |
| 11 | Grammar Highlight Demo | Prove VS Code grammar works. | • `.raven` syntax variety • VSCode extension | `examples/highlight` + `vscode-raven/` | Highlighting screenshot included | Add semantic tokens |
| 12 | Benchmark / Size Comparison | Compare output size vs baseline. | • CLI `build --release` • JS minifier stats | `examples/benchmark` | Report original vs minified sizes in README | Automate benchmark script |
| 13 | Reactive Counter | Show incremental state updates even if static. | • Signals placeholder • Annotations | `examples/counter` | Build emits JS stub that logs state change TODO | Wire to runtime once ready |
| 14 | Server Call Mock | Illustrate `@server` annotation separation. | • `@server` vs client • RPC stubs | `examples/server-call` | Build splits server/client bundles and logs call boundary | Add actual HTTP stub |
| 15 | Form Handling | Demonstrate simple form submission. | • `<form>` tags • Event handler placeholders | `examples/forms` | Generated HTML includes form with attributes | Connect to runtime validation later |
| 16 | Data Table | Render table with formatting helpers. | • Loops • Helper functions | `examples/table` | Table rows render correctly from array | Add sortable headers |
| 17 | Router Skeleton | Show planned routing concept. | • Placeholder router config • Multiple components | `examples/router` | README explains routing TODOs; build outputs static fallback | Integrate dev server routing |
| 18 | 5-Minute Tutorial App | Mirror tutorial steps for documentation. | • CLI `init/dev/build` • Basic component | `examples/tutorial-app` | Tutorial instructions produce matching output | Add video walkthrough |
| 19 | Syntax Playground (Web) | Host editable textarea that compiles on demand (mock). | • CLI compile API • Dev server integration | `examples/playground-web` | README describes workflow; dev server serves static assets | Integrate WASM worker when ready |
| 20 | Mini Dashboard | Present cards/charts layout using static data. | • Components • Lists • Attributes | `examples/dashboard` | Build outputs dashboard HTML for screenshot | Hook up chart JS stub |

## Compiler Stage Coverage

| Project # | Lex | Parse | AST | Codegen | CLI | Dev Server | VS Code |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| 2 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| 3 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| 4 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| 5 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| 6 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 7 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 8 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 9 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 10 | ✅ | ✅ | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| 11 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ✅ |
| 12 | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| 13 | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| 14 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 15 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 16 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 17 | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ⚠️ |
| 18 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| 19 | ✅ | ✅ | ✅ | ⚠️ | ✅ | ✅ | ✅ |
| 20 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ |

Legend: ✅ = primary focus, ⚠️ = partial / planned coverage.
