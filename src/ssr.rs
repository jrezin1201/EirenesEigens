// Server-Side Rendering (SSR) Module
// Renders components to HTML on the server

use crate::vdom::VNode;
use std::collections::HashMap;

/// SSR Context - holds server-side state during rendering
pub struct SSRContext {
    pub metadata: HashMap<String, String>,
    pub head_elements: Vec<String>,
    pub preload_scripts: Vec<String>,
}

impl SSRContext {
    pub fn new() -> Self {
        SSRContext {
            metadata: HashMap::new(),
            head_elements: Vec::new(),
            preload_scripts: Vec::new(),
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.metadata.insert("title".to_string(), title.to_string());
    }

    pub fn add_meta(&mut self, name: &str, content: &str) {
        self.head_elements.push(format!(
            r#"<meta name="{}" content="{}">"#,
            escape_html(name),
            escape_html(content)
        ));
    }

    pub fn add_preload_script(&mut self, src: &str) {
        self.preload_scripts.push(src.to_string());
    }
}

impl Default for SSRContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a VNode tree to HTML string
pub fn render_to_string(vnode: &VNode, ctx: &mut SSRContext) -> String {
    match vnode {
        VNode::Element { tag, attrs, children } => {
            let mut html = String::new();

            // Opening tag
            html.push('<');
            html.push_str(tag);

            // Attributes
            for (key, value) in attrs {
                html.push(' ');
                html.push_str(key);
                html.push_str(r#"=""#);
                html.push_str(&escape_html(value));
                html.push('"');
            }

            // Self-closing tags
            if children.is_empty() && is_void_element(tag) {
                html.push_str(" />");
                return html;
            }

            html.push('>');

            // Children
            for child in children {
                html.push_str(&render_to_string(child, ctx));
            }

            // Closing tag
            html.push_str("</");
            html.push_str(tag);
            html.push('>');

            html
        }
        VNode::Text(content) => escape_html(content),
    }
}

/// Render a complete HTML document with hydration support
pub fn render_to_document(
    vnode: &VNode,
    ctx: &mut SSRContext,
    app_name: &str,
) -> String {
    let body_html = render_to_string(vnode, ctx);
    let default_title = app_name.to_string();
    let title = ctx.metadata.get("title").unwrap_or(&default_title);

    let mut doc = String::new();
    doc.push_str("<!DOCTYPE html>\n");
    doc.push_str("<html lang=\"en\">\n");
    doc.push_str("<head>\n");
    doc.push_str("  <meta charset=\"UTF-8\">\n");
    doc.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    doc.push_str(&format!("  <title>{}</title>\n", escape_html(title)));

    // Add custom head elements
    for elem in &ctx.head_elements {
        doc.push_str("  ");
        doc.push_str(elem);
        doc.push('\n');
    }

    doc.push_str("</head>\n");
    doc.push_str("<body>\n");
    doc.push_str("  <div id=\"app\">\n");
    doc.push_str(&indent_html(&body_html, 2));
    doc.push_str("  </div>\n");

    // Add hydration script
    doc.push_str("  <script>\n");
    doc.push_str("    // Hydration data\n");
    doc.push_str("    window.__INITIAL_STATE__ = {};\n");
    doc.push_str("  </script>\n");

    // Add preload scripts
    for script in &ctx.preload_scripts {
        doc.push_str(&format!("  <script src=\"{}\" defer></script>\n", escape_html(script)));
    }

    doc.push_str("</body>\n");
    doc.push_str("</html>");

    doc
}

/// Render with streaming support (for large pages)
pub struct SSRStream {
    buffer: String,
    flushed: usize,
}

impl SSRStream {
    pub fn new() -> Self {
        SSRStream {
            buffer: String::new(),
            flushed: 0,
        }
    }

    pub fn write(&mut self, html: &str) {
        self.buffer.push_str(html);
    }

    pub fn flush(&mut self) -> String {
        let chunk = self.buffer[self.flushed..].to_string();
        self.flushed = self.buffer.len();
        chunk
    }

    pub fn render_streaming(&mut self, vnode: &VNode, ctx: &mut SSRContext) -> String {
        let html = render_to_string(vnode, ctx);
        self.write(&html);
        self.flush()
    }
}

impl Default for SSRStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape HTML special characters
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Check if an element is void (self-closing)
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
        | "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
}

/// Indent HTML for pretty printing
fn indent_html(html: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    html.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate hydration markers for client-side takeover
pub fn generate_hydration_id() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("h{}", id)
}

/// SSR Performance metrics
#[derive(Debug, Clone, Default)]
pub struct SSRMetrics {
    pub render_time_ms: u64,
    pub html_size_bytes: usize,
    pub components_rendered: usize,
}

impl SSRMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_render(&mut self, start_time: std::time::Instant) {
        self.render_time_ms = start_time.elapsed().as_millis() as u64;
    }

    pub fn record_html_size(&mut self, html: &str) {
        self.html_size_bytes = html.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("'quote'"), "&#39;quote&#39;");
    }

    #[test]
    fn test_render_simple_element() {
        let vnode = VNode::Element {
            tag: "div".to_string(),
            attrs: vec![("class".to_string(), "container".to_string())],
            children: vec![VNode::Text("Hello".to_string())],
        };

        let mut ctx = SSRContext::new();
        let html = render_to_string(&vnode, &mut ctx);
        assert_eq!(html, r#"<div class="container">Hello</div>"#);
    }

    #[test]
    fn test_render_void_element() {
        let vnode = VNode::Element {
            tag: "br".to_string(),
            attrs: vec![],
            children: vec![],
        };

        let mut ctx = SSRContext::new();
        let html = render_to_string(&vnode, &mut ctx);
        assert_eq!(html, "<br />");
    }

    #[test]
    fn test_render_nested_elements() {
        let vnode = VNode::Element {
            tag: "div".to_string(),
            attrs: vec![],
            children: vec![
                VNode::Element {
                    tag: "h1".to_string(),
                    attrs: vec![],
                    children: vec![VNode::Text("Title".to_string())],
                },
                VNode::Element {
                    tag: "p".to_string(),
                    attrs: vec![],
                    children: vec![VNode::Text("Content".to_string())],
                },
            ],
        };

        let mut ctx = SSRContext::new();
        let html = render_to_string(&vnode, &mut ctx);
        assert_eq!(html, "<div><h1>Title</h1><p>Content</p></div>");
    }
}
