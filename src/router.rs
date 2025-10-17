// Router System for RavensOne
// Client-side routing with history API, dynamic routes, and nested routing

use crate::reactive::{Signal, create_effect};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Route parameter type
pub type RouteParams = HashMap<String, String>;

/// Route definition
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub component: String, // Component name to render
    pub meta: RouteMetadata,
}

/// Route metadata
#[derive(Debug, Clone, Default)]
pub struct RouteMetadata {
    pub title: Option<String>,
    pub requires_auth: bool,
    pub layout: Option<String>,
}

impl Route {
    pub fn new(path: &str, component: &str) -> Self {
        Route {
            path: path.to_string(),
            component: component.to_string(),
            meta: RouteMetadata::default(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.meta.title = Some(title.to_string());
        self
    }

    pub fn requires_auth(mut self) -> Self {
        self.meta.requires_auth = true;
        self
    }

    pub fn with_layout(mut self, layout: &str) -> Self {
        self.meta.layout = Some(layout.to_string());
        self
    }
}

/// Router configuration
pub struct Router {
    routes: Vec<Route>,
    current_route: Signal<Option<MatchedRoute>>,
    base_path: String,
    mode: RouterMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RouterMode {
    /// Uses HTML5 History API (clean URLs)
    History,
    /// Uses URL hash (# URLs, works without server config)
    Hash,
}

/// Matched route with extracted parameters
#[derive(Debug, Clone)]
pub struct MatchedRoute {
    pub route: Route,
    pub params: RouteParams,
    pub query: HashMap<String, String>,
    pub path: String,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            current_route: Signal::new(None),
            base_path: String::from("/"),
            mode: RouterMode::History,
        }
    }

    /// Set router mode
    pub fn mode(mut self, mode: RouterMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set base path for all routes
    pub fn base_path(mut self, path: &str) -> Self {
        self.base_path = path.to_string();
        self
    }

    /// Add a route
    pub fn route(mut self, path: &str, component: &str) -> Self {
        self.routes.push(Route::new(path, component));
        self
    }

    /// Add a route with configuration
    pub fn add_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    /// Navigate to a path
    pub fn push(&self, path: &str) {
        match self.mode {
            RouterMode::History => {
                // In a real implementation, would use window.history.pushState
                println!("[Router] Navigating to: {}", path);
            }
            RouterMode::Hash => {
                println!("[Router] Navigating to: #{}", path);
            }
        }

        // Match the route and update current_route
        if let Some(matched) = self.match_route(path) {
            self.current_route.set(Some(matched));
        }
    }

    /// Replace current route (no history entry)
    pub fn replace(&self, path: &str) {
        println!("[Router] Replacing with: {}", path);
        if let Some(matched) = self.match_route(path) {
            self.current_route.set(Some(matched));
        }
    }

    /// Go back in history
    pub fn back(&self) {
        println!("[Router] Going back");
        // In real implementation: window.history.back()
    }

    /// Go forward in history
    pub fn forward(&self) {
        println!("[Router] Going forward");
        // In real implementation: window.history.forward()
    }

    /// Match a path to a route
    fn match_route(&self, path: &str) -> Option<MatchedRoute> {
        let (clean_path, query) = self.parse_path(path);

        for route in &self.routes {
            if let Some(params) = self.match_pattern(&route.path, &clean_path) {
                return Some(MatchedRoute {
                    route: route.clone(),
                    params,
                    query,
                    path: clean_path.clone(),
                });
            }
        }

        None
    }

    /// Parse path and extract query parameters
    fn parse_path(&self, path: &str) -> (String, HashMap<String, String>) {
        let parts: Vec<&str> = path.split('?').collect();
        let clean_path = parts[0].to_string();
        let mut query = HashMap::new();

        if parts.len() > 1 {
            for pair in parts[1].split('&') {
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    query.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }

        (clean_path, query)
    }

    /// Match a route pattern against a path
    fn match_pattern(&self, pattern: &str, path: &str) -> Option<RouteParams> {
        let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Different number of segments = no match
        if pattern_segments.len() != path_segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
            if pattern_seg.starts_with(':') {
                // Dynamic segment - extract parameter
                let param_name = &pattern_seg[1..];
                params.insert(param_name.to_string(), path_seg.to_string());
            } else if pattern_seg != path_seg {
                // Static segment doesn't match
                return None;
            }
        }

        Some(params)
    }

    /// Get current route
    pub fn current(&self) -> Option<MatchedRoute> {
        self.current_route.get()
    }

    /// Initialize router (start listening to browser events)
    pub fn init(&self) {
        println!("[Router] Initialized with mode: {:?}", self.mode);

        // In a real implementation, would set up popstate listener:
        // window.addEventListener('popstate', |event| {
        //     let path = window.location.pathname;
        //     self.push(path);
        // });

        // Match initial route
        let initial_path = "/"; // In real: window.location.pathname
        self.push(initial_path);
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Router Link component helper
#[derive(Debug, Clone)]
pub struct Link {
    pub to: String,
    pub replace: bool,
    pub active_class: Option<String>,
}

impl Link {
    pub fn new(to: &str) -> Self {
        Link {
            to: to.to_string(),
            replace: false,
            active_class: None,
        }
    }

    pub fn replace(mut self) -> Self {
        self.replace = true;
        self
    }

    pub fn active_class(mut self, class: &str) -> Self {
        self.active_class = Some(class.to_string());
        self
    }
}

/// Navigation guard - runs before route changes
pub type NavigationGuard = Box<dyn Fn(&MatchedRoute) -> bool>;

/// Router with guards
pub struct GuardedRouter {
    router: Router,
    before_guards: Vec<NavigationGuard>,
}

impl GuardedRouter {
    pub fn new(router: Router) -> Self {
        GuardedRouter {
            router,
            before_guards: Vec::new(),
        }
    }

    /// Add a before-navigation guard
    pub fn before_each<F>(mut self, guard: F) -> Self
    where
        F: Fn(&MatchedRoute) -> bool + 'static,
    {
        self.before_guards.push(Box::new(guard));
        self
    }

    /// Navigate with guards
    pub fn push(&self, path: &str) {
        // Check guards before navigating
        if let Some(matched) = self.router.match_route(path) {
            for guard in &self.before_guards {
                if !guard(&matched) {
                    println!("[Router] Navigation blocked by guard");
                    return;
                }
            }
            self.router.push(path);
        }
    }
}

/// Nested router for sub-routes
pub struct NestedRouter {
    parent_path: String,
    router: Router,
}

impl NestedRouter {
    pub fn new(parent_path: &str) -> Self {
        NestedRouter {
            parent_path: parent_path.to_string(),
            router: Router::new(),
        }
    }

    /// Add a child route (automatically prefixes with parent path)
    pub fn route(mut self, path: &str, component: &str) -> Self {
        let full_path = format!("{}{}", self.parent_path, path);
        self.router = self.router.route(&full_path, component);
        self
    }
}

/// Route builder for complex configurations
pub struct RouteBuilder {
    route: Route,
    children: Vec<Route>,
}

impl RouteBuilder {
    pub fn new(path: &str, component: &str) -> Self {
        RouteBuilder {
            route: Route::new(path, component),
            children: Vec::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.route = self.route.with_title(title);
        self
    }

    pub fn requires_auth(mut self) -> Self {
        self.route = self.route.requires_auth();
        self
    }

    pub fn child(mut self, path: &str, component: &str) -> Self {
        let child_path = format!("{}{}", self.route.path, path);
        self.children.push(Route::new(&child_path, component));
        self
    }

    pub fn build(self) -> Vec<Route> {
        let mut routes = vec![self.route];
        routes.extend(self.children);
        routes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_route_matching() {
        let router = Router::new()
            .route("/", "Home")
            .route("/about", "About")
            .route("/contact", "Contact");

        let matched = router.match_route("/about");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().route.component, "About");
    }

    #[test]
    fn test_dynamic_route_matching() {
        let router = Router::new()
            .route("/users/:id", "UserProfile")
            .route("/posts/:slug", "Post");

        let matched = router.match_route("/users/123");
        assert!(matched.is_some());

        let route = matched.unwrap();
        assert_eq!(route.route.component, "UserProfile");
        assert_eq!(route.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_query_parameters() {
        let router = Router::new().route("/search", "Search");

        let (path, query) = router.parse_path("/search?q=rust&page=2");
        assert_eq!(path, "/search");
        assert_eq!(query.get("q"), Some(&"rust".to_string()));
        assert_eq!(query.get("page"), Some(&"2".to_string()));
    }

    #[test]
    fn test_multiple_params() {
        let router = Router::new()
            .route("/blog/:year/:month/:slug", "BlogPost");

        let matched = router.match_route("/blog/2025/10/hello-world");
        assert!(matched.is_some());

        let route = matched.unwrap();
        assert_eq!(route.params.get("year"), Some(&"2025".to_string()));
        assert_eq!(route.params.get("month"), Some(&"10".to_string()));
        assert_eq!(route.params.get("slug"), Some(&"hello-world".to_string()));
    }

    #[test]
    fn test_no_match() {
        let router = Router::new()
            .route("/", "Home")
            .route("/about", "About");

        let matched = router.match_route("/nonexistent");
        assert!(matched.is_none());
    }

    #[test]
    fn test_route_builder() {
        let routes = RouteBuilder::new("/dashboard", "Dashboard")
            .title("Dashboard")
            .requires_auth()
            .child("/settings", "Settings")
            .child("/profile", "Profile")
            .build();

        assert_eq!(routes.len(), 3);
        assert_eq!(routes[0].path, "/dashboard");
        assert_eq!(routes[1].path, "/dashboard/settings");
        assert_eq!(routes[2].path, "/dashboard/profile");
    }
}
