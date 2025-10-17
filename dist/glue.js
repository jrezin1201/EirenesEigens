// src/vdom.rs

// Represents a node in the Virtual DOM. It can be an element or plain text.
#[derive(Debug, Clone)]
pub enum VNode {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<VNode>,
    },
    Text(String),
}

// Represents a single, minimal change that needs to be made to the real DOM.
#[derive(Debug, Clone)]
pub enum Patch {
    CreateElement { tag: String, id: usize },
    SetText { id: usize, content: String },
    SetAttribute { id: usize, name: String, value: String },
    AppendChild { parent_id: usize, child_id: usize },
}

/// The core diffing algorithm.
/// It compares the new VDOM tree to the old one and generates a list of patches.
pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    // This is a simplified diffing algorithm. A real one would be much more complex,
    // handling keyed lists, component updates, etc.
    
    // For now, we'll just replace the entire tree.
    // 1. Create the new root element.
    if let VNode::Element { tag, .. } = new {
        patches.push(Patch::CreateElement { tag: tag.clone(), id: 0 });
    }

    // 2. Recursively add children.
    // ... logic to traverse children and create SetText and AppendChild patches ...

    patches
}