// Client-Side Hydration Module
// Takes over server-rendered HTML and makes it interactive

use crate::vdom::VNode;
use std::collections::HashMap;

/// Hydration state - tracks which nodes have been hydrated
#[derive(Debug, Clone)]
pub struct HydrationState {
    pub hydrated_nodes: HashMap<String, bool>,
    pub mismatches: Vec<HydrationMismatch>,
}

#[derive(Debug, Clone)]
pub struct HydrationMismatch {
    pub node_id: String,
    pub expected: String,
    pub actual: String,
}

impl HydrationState {
    pub fn new() -> Self {
        HydrationState {
            hydrated_nodes: HashMap::new(),
            mismatches: Vec::new(),
        }
    }

    pub fn mark_hydrated(&mut self, node_id: String) {
        self.hydrated_nodes.insert(node_id, true);
    }

    pub fn record_mismatch(&mut self, node_id: String, expected: String, actual: String) {
        self.mismatches.push(HydrationMismatch {
            node_id,
            expected,
            actual,
        });
    }

    pub fn is_hydrated(&self, node_id: &str) -> bool {
        self.hydrated_nodes.get(node_id).copied().unwrap_or(false)
    }

    pub fn has_mismatches(&self) -> bool {
        !self.mismatches.is_empty()
    }
}

impl Default for HydrationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Hydration strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HydrationStrategy {
    /// Hydrate immediately on page load
    Immediate,
    /// Hydrate when element becomes visible (intersection observer)
    WhenVisible,
    /// Hydrate on user interaction
    OnInteraction,
    /// Hydrate after a delay
    Delayed(u32), // milliseconds
}

/// Hydration plan - determines how to hydrate components
#[derive(Debug, Clone)]
pub struct HydrationPlan {
    pub strategy: HydrationStrategy,
    pub priority: u8, // 0-255, higher = more important
    pub dependencies: Vec<String>, // Component IDs that must be hydrated first
}

impl HydrationPlan {
    pub fn immediate() -> Self {
        HydrationPlan {
            strategy: HydrationStrategy::Immediate,
            priority: 255,
            dependencies: Vec::new(),
        }
    }

    pub fn lazy() -> Self {
        HydrationPlan {
            strategy: HydrationStrategy::WhenVisible,
            priority: 128,
            dependencies: Vec::new(),
        }
    }

    pub fn on_interaction() -> Self {
        HydrationPlan {
            strategy: HydrationStrategy::OnInteraction,
            priority: 64,
            dependencies: Vec::new(),
        }
    }
}

/// Hydrate a virtual DOM tree against existing HTML
pub fn hydrate_tree(vnode: &VNode, state: &mut HydrationState) -> Result<(), String> {
    match vnode {
        VNode::Element { tag, attrs, children } => {
            // Generate hydration ID
            let node_id = format!("{}-{}", tag, attrs.len());

            // Check if already hydrated
            if state.is_hydrated(&node_id) {
                return Ok(());
            }

            // Validate structure matches
            // In a real implementation, we'd check the DOM here

            // Hydrate children
            for child in children {
                hydrate_tree(child, state)?;
            }

            // Mark as hydrated
            state.mark_hydrated(node_id);
            Ok(())
        }
        VNode::Text(_content) => {
            // Text nodes are automatically hydrated
            Ok(())
        }
    }
}

/// Generate hydration script for client
pub fn generate_hydration_script(state: &HydrationState) -> String {
    let mut script = String::from("(function() {\n");
    script.push_str("  'use strict';\n");
    script.push_str("  \n");
    script.push_str("  // Hydration runtime\n");
    script.push_str("  window.__RAVEN_HYDRATE__ = function(componentId) {\n");
    script.push_str("    const el = document.querySelector('[data-component=\"' + componentId + '\"]');\n");
    script.push_str("    if (!el) return;\n");
    script.push_str("    \n");
    script.push_str("    // Attach event listeners\n");
    script.push_str("    el.querySelectorAll('[data-event]').forEach(function(node) {\n");
    script.push_str("      const event = node.getAttribute('data-event');\n");
    script.push_str("      const handler = node.getAttribute('data-handler');\n");
    script.push_str("      if (event && handler && window[handler]) {\n");
    script.push_str("        node.addEventListener(event, window[handler]);\n");
    script.push_str("      }\n");
    script.push_str("    });\n");
    script.push_str("    \n");
    script.push_str("    el.setAttribute('data-hydrated', 'true');\n");
    script.push_str("  };\n");
    script.push_str("  \n");

    // Add hydrated node IDs
    script.push_str("  // Already hydrated nodes\n");
    script.push_str("  window.__HYDRATED_NODES__ = ");
    script.push_str(&serde_json::to_string(&state.hydrated_nodes).unwrap_or_else(|_| "{}".to_string()));
    script.push_str(";\n");

    script.push_str("})();\n");
    script
}

/// Progressive hydration scheduler
pub struct HydrationScheduler {
    pending: Vec<(String, HydrationPlan)>,
    hydrated: Vec<String>,
}

impl HydrationScheduler {
    pub fn new() -> Self {
        HydrationScheduler {
            pending: Vec::new(),
            hydrated: Vec::new(),
        }
    }

    pub fn schedule(&mut self, component_id: String, plan: HydrationPlan) {
        self.pending.push((component_id, plan));
        // Sort by priority (higher first)
        self.pending.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
    }

    pub fn next(&mut self) -> Option<(String, HydrationPlan)> {
        if self.pending.is_empty() {
            return None;
        }

        // Find next component whose dependencies are satisfied
        for i in 0..self.pending.len() {
            let (component_id, plan) = &self.pending[i];
            let deps_satisfied = plan.dependencies.iter()
                .all(|dep| self.hydrated.contains(dep));

            if deps_satisfied {
                let result = self.pending.remove(i);
                self.hydrated.push(result.0.clone());
                return Some(result);
            }
        }

        // If no dependencies satisfied, take highest priority
        if !self.pending.is_empty() {
            let result = self.pending.remove(0);
            self.hydrated.push(result.0.clone());
            return Some(result);
        }

        None
    }

    pub fn is_complete(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Default for HydrationScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_state() {
        let mut state = HydrationState::new();
        assert!(!state.is_hydrated("test-node"));

        state.mark_hydrated("test-node".to_string());
        assert!(state.is_hydrated("test-node"));
    }

    #[test]
    fn test_hydration_scheduler() {
        let mut scheduler = HydrationScheduler::new();

        scheduler.schedule("component-a".to_string(), HydrationPlan {
            strategy: HydrationStrategy::Immediate,
            priority: 100,
            dependencies: vec![],
        });

        scheduler.schedule("component-b".to_string(), HydrationPlan {
            strategy: HydrationStrategy::Immediate,
            priority: 200,
            dependencies: vec![],
        });

        // Should get component-b first (higher priority)
        let (id, _) = scheduler.next().unwrap();
        assert_eq!(id, "component-b");

        let (id, _) = scheduler.next().unwrap();
        assert_eq!(id, "component-a");

        assert!(scheduler.is_complete());
    }

    #[test]
    fn test_hydration_with_dependencies() {
        let mut scheduler = HydrationScheduler::new();

        scheduler.schedule("child".to_string(), HydrationPlan {
            strategy: HydrationStrategy::Immediate,
            priority: 255,
            dependencies: vec!["parent".to_string()],
        });

        scheduler.schedule("parent".to_string(), HydrationPlan {
            strategy: HydrationStrategy::Immediate,
            priority: 100,
            dependencies: vec![],
        });

        // Should get parent first (dependency of child)
        let (id, _) = scheduler.next().unwrap();
        assert_eq!(id, "parent");

        let (id, _) = scheduler.next().unwrap();
        assert_eq!(id, "child");
    }
}
