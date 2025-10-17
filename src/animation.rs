// Animation System for RavensOne
// CSS transitions, keyframe animations, spring physics, and gesture support

use crate::reactive::Signal;
use std::collections::HashMap;

/// Easing functions for animations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    Spring,
    Bounce,
}

impl Easing {
    /// Get the CSS timing function for this easing
    pub fn to_css(&self) -> &str {
        match self {
            Easing::Linear => "linear",
            Easing::EaseIn => "ease-in",
            Easing::EaseOut => "ease-out",
            Easing::EaseInOut => "ease-in-out",
            Easing::EaseInQuad => "cubic-bezier(0.55, 0.085, 0.68, 0.53)",
            Easing::EaseOutQuad => "cubic-bezier(0.25, 0.46, 0.45, 0.94)",
            Easing::EaseInOutQuad => "cubic-bezier(0.455, 0.03, 0.515, 0.955)",
            Easing::EaseInCubic => "cubic-bezier(0.55, 0.055, 0.675, 0.19)",
            Easing::EaseOutCubic => "cubic-bezier(0.215, 0.61, 0.355, 1)",
            Easing::EaseInOutCubic => "cubic-bezier(0.645, 0.045, 0.355, 1)",
            Easing::EaseInQuart => "cubic-bezier(0.895, 0.03, 0.685, 0.22)",
            Easing::EaseOutQuart => "cubic-bezier(0.165, 0.84, 0.44, 1)",
            Easing::EaseInOutQuart => "cubic-bezier(0.77, 0, 0.175, 1)",
            Easing::Spring => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
            Easing::Bounce => "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
        }
    }

    /// Calculate the value at a given progress (0.0 to 1.0)
    pub fn calculate(&self, progress: f64) -> f64 {
        match self {
            Easing::Linear => progress,
            Easing::EaseIn => progress * progress,
            Easing::EaseOut => progress * (2.0 - progress),
            Easing::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    -1.0 + (4.0 - 2.0 * progress) * progress
                }
            }
            Easing::EaseInQuad => progress * progress,
            Easing::EaseOutQuad => progress * (2.0 - progress),
            Easing::EaseInOutQuad => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    -1.0 + (4.0 - 2.0 * progress) * progress
                }
            }
            Easing::EaseInCubic => progress * progress * progress,
            Easing::EaseOutCubic => {
                let p = progress - 1.0;
                p * p * p + 1.0
            }
            Easing::EaseInOutCubic => {
                if progress < 0.5 {
                    4.0 * progress * progress * progress
                } else {
                    let p = 2.0 * progress - 2.0;
                    1.0 + p * p * p / 2.0
                }
            }
            Easing::EaseInQuart => progress * progress * progress * progress,
            Easing::EaseOutQuart => {
                let p = progress - 1.0;
                1.0 - p * p * p * p
            }
            Easing::EaseInOutQuart => {
                if progress < 0.5 {
                    8.0 * progress * progress * progress * progress
                } else {
                    let p = progress - 1.0;
                    1.0 - 8.0 * p * p * p * p
                }
            }
            Easing::Spring | Easing::Bounce => {
                // Simplified spring/bounce calculation
                let p = progress;
                p * p * (2.7 * p - 1.7)
            }
        }
    }
}

/// Animation configuration
#[derive(Clone)]
pub struct Animation {
    pub property: String,
    pub from: f64,
    pub to: f64,
    pub duration: u32, // milliseconds
    pub delay: u32,    // milliseconds
    pub easing: Easing,
    pub repeat: RepeatMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepeatMode {
    None,
    Loop,
    PingPong,
    Count(u32),
}

impl Animation {
    pub fn new(property: &str, from: f64, to: f64, duration: u32) -> Self {
        Animation {
            property: property.to_string(),
            from,
            to,
            duration,
            delay: 0,
            easing: Easing::EaseInOut,
            repeat: RepeatMode::None,
        }
    }

    pub fn delay(mut self, delay_ms: u32) -> Self {
        self.delay = delay_ms;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn repeat(mut self, mode: RepeatMode) -> Self {
        self.repeat = mode;
        self
    }

    /// Generate CSS for this animation
    pub fn to_css(&self) -> String {
        format!(
            "{} {}ms {} {}ms",
            self.property,
            self.duration,
            self.easing.to_css(),
            self.delay
        )
    }

    /// Calculate the current value at a given time
    pub fn value_at(&self, elapsed: u32) -> f64 {
        if elapsed < self.delay {
            return self.from;
        }

        let progress = ((elapsed - self.delay) as f64 / self.duration as f64).min(1.0);
        let eased = self.easing.calculate(progress);
        self.from + (self.to - self.from) * eased
    }
}

/// Transition helper for common animations
pub struct Transition {
    pub property: String,
    pub duration: u32,
    pub easing: Easing,
}

impl Transition {
    pub fn new(property: &str, duration: u32) -> Self {
        Transition {
            property: property.to_string(),
            duration,
            easing: Easing::EaseInOut,
        }
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Generate CSS transition string
    pub fn to_css(&self) -> String {
        format!(
            "{} {}ms {}",
            self.property,
            self.duration,
            self.easing.to_css()
        )
    }
}

/// Keyframe animation
pub struct Keyframes {
    pub name: String,
    pub frames: Vec<Keyframe>,
}

#[derive(Clone)]
pub struct Keyframe {
    pub offset: f64, // 0.0 to 1.0 (percentage)
    pub properties: HashMap<String, String>,
}

impl Keyframes {
    pub fn new(name: &str) -> Self {
        Keyframes {
            name: name.to_string(),
            frames: Vec::new(),
        }
    }

    pub fn add_frame(mut self, offset: f64, properties: HashMap<String, String>) -> Self {
        self.frames.push(Keyframe { offset, properties });
        self
    }

    /// Generate CSS @keyframes rule
    pub fn to_css(&self) -> String {
        let mut css = format!("@keyframes {} {{\n", self.name);

        for frame in &self.frames {
            let percent = (frame.offset * 100.0) as u32;
            css.push_str(&format!("  {}% {{\n", percent));

            for (prop, value) in &frame.properties {
                css.push_str(&format!("    {}: {};\n", prop, value));
            }

            css.push_str("  }\n");
        }

        css.push_str("}\n");
        css
    }
}

/// Spring physics animation
pub struct Spring {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}

impl Spring {
    pub fn new() -> Self {
        Spring {
            stiffness: 100.0,
            damping: 10.0,
            mass: 1.0,
        }
    }

    pub fn stiff() -> Self {
        Spring {
            stiffness: 300.0,
            damping: 20.0,
            mass: 1.0,
        }
    }

    pub fn gentle() -> Self {
        Spring {
            stiffness: 50.0,
            damping: 15.0,
            mass: 1.0,
        }
    }

    pub fn wobbly() -> Self {
        Spring {
            stiffness: 180.0,
            damping: 8.0,
            mass: 1.0,
        }
    }

    /// Calculate spring value at time t
    pub fn value_at(&self, from: f64, to: f64, velocity: f64, time: f64) -> f64 {
        let displacement = from - to;
        let omega = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.stiffness * self.mass).sqrt());

        if zeta < 1.0 {
            // Underdamped (oscillation)
            let omega_d = omega * (1.0 - zeta * zeta).sqrt();
            let a = displacement;
            let b = (velocity + zeta * omega * displacement) / omega_d;
            let envelope = (-zeta * omega * time).exp();

            to + envelope * (a * (omega_d * time).cos() + b * (omega_d * time).sin())
        } else if zeta == 1.0 {
            // Critically damped
            let a = displacement;
            let b = velocity + omega * displacement;
            to + (a + b * time) * (-omega * time).exp()
        } else {
            // Overdamped
            let omega1 = omega * (zeta + (zeta * zeta - 1.0).sqrt());
            let omega2 = omega * (zeta - (zeta * zeta - 1.0).sqrt());
            let a = (velocity - omega2 * displacement) / (omega1 - omega2);
            let b = displacement - a;

            to + a * (-omega1 * time).exp() + b * (-omega2 * time).exp()
        }
    }
}

impl Default for Spring {
    fn default() -> Self {
        Self::new()
    }
}

/// Animated value with reactive updates
pub struct AnimatedValue {
    pub current: Signal<f64>,
    target: f64,
    velocity: f64,
    animation: Option<Animation>,
}

impl AnimatedValue {
    pub fn new(initial: f64) -> Self {
        AnimatedValue {
            current: Signal::new(initial),
            target: initial,
            velocity: 0.0,
            animation: None,
        }
    }

    /// Animate to a new value
    pub fn animate_to(&mut self, target: f64, duration: u32, easing: Easing) {
        let from = self.current.get();
        self.target = target;

        self.animation = Some(Animation {
            property: "value".to_string(),
            from,
            to: target,
            duration,
            delay: 0,
            easing,
            repeat: RepeatMode::None,
        });
    }

    /// Spring to a new value
    pub fn spring_to(&mut self, target: f64) {
        self.target = target;
        // In a real implementation, this would use spring physics
        // For now, use a spring-like easing
        self.animate_to(target, 500, Easing::Spring);
    }

    /// Update the animation (called each frame)
    pub fn tick(&mut self, elapsed: u32) {
        if let Some(anim) = &self.animation {
            let value = anim.value_at(elapsed);
            self.current.set(value);

            // Stop animation when complete
            if elapsed >= anim.duration + anim.delay {
                self.animation = None;
            }
        }
    }

    /// Get current value
    pub fn get(&self) -> f64 {
        self.current.get()
    }
}

/// Gesture recognizer for animations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    Tap,
    DoubleTap,
    LongPress,
    Swipe(Direction),
    Pan(Direction),
    Pinch,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Animated transition group
pub struct TransitionGroup {
    pub enter: Vec<Animation>,
    pub exit: Vec<Animation>,
    pub duration: u32,
}

impl TransitionGroup {
    pub fn new() -> Self {
        TransitionGroup {
            enter: Vec::new(),
            exit: Vec::new(),
            duration: 300,
        }
    }

    pub fn fade() -> Self {
        TransitionGroup {
            enter: vec![Animation::new("opacity", 0.0, 1.0, 300)],
            exit: vec![Animation::new("opacity", 1.0, 0.0, 300)],
            duration: 300,
        }
    }

    pub fn slide_up() -> Self {
        TransitionGroup {
            enter: vec![
                Animation::new("transform", 100.0, 0.0, 300),
                Animation::new("opacity", 0.0, 1.0, 300),
            ],
            exit: vec![
                Animation::new("transform", 0.0, -100.0, 300),
                Animation::new("opacity", 1.0, 0.0, 300),
            ],
            duration: 300,
        }
    }

    pub fn scale() -> Self {
        TransitionGroup {
            enter: vec![Animation::new("scale", 0.8, 1.0, 300)],
            exit: vec![Animation::new("scale", 1.0, 0.8, 300)],
            duration: 300,
        }
    }
}

impl Default for TransitionGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Stagger animation helper
pub struct StaggerAnimation {
    pub base_animation: Animation,
    pub stagger_delay: u32, // delay between each item
}

impl StaggerAnimation {
    pub fn new(animation: Animation, stagger_delay: u32) -> Self {
        StaggerAnimation {
            base_animation: animation,
            stagger_delay,
        }
    }

    /// Get animation for nth item
    pub fn for_index(&self, index: usize) -> Animation {
        let mut anim = self.base_animation.clone();
        anim.delay = self.stagger_delay * index as u32;
        anim
    }
}

/// Parallax effect
pub struct Parallax {
    pub speed: f64, // 0.0 to 1.0, where 1.0 is normal speed
}

impl Parallax {
    pub fn new(speed: f64) -> Self {
        Parallax {
            speed: speed.max(0.0).min(1.0),
        }
    }

    /// Calculate offset based on scroll position
    pub fn calculate_offset(&self, scroll_position: f64) -> f64 {
        scroll_position * self.speed
    }
}

/// Morphing animation between shapes
pub struct Morph {
    pub from_path: String,
    pub to_path: String,
    pub duration: u32,
    pub easing: Easing,
}

impl Morph {
    pub fn new(from: &str, to: &str, duration: u32) -> Self {
        Morph {
            from_path: from.to_string(),
            to_path: to.to_string(),
            duration,
            easing: Easing::EaseInOut,
        }
    }

    /// Generate SVG animate element
    pub fn to_svg_animate(&self) -> String {
        format!(
            r#"<animate attributeName="d" from="{}" to="{}" dur="{}ms" />"#,
            self.from_path, self.to_path, self.duration
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let easing = Easing::Linear;
        assert_eq!(easing.calculate(0.0), 0.0);
        assert_eq!(easing.calculate(0.5), 0.5);
        assert_eq!(easing.calculate(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = Easing::EaseIn;
        assert_eq!(easing.calculate(0.0), 0.0);
        assert!(easing.calculate(0.5) < 0.5); // Should be slower at start
        assert_eq!(easing.calculate(1.0), 1.0);
    }

    #[test]
    fn test_animation_value_at() {
        let anim = Animation::new("opacity", 0.0, 1.0, 1000);

        assert_eq!(anim.value_at(0), 0.0);
        assert!(anim.value_at(500) > 0.0 && anim.value_at(500) < 1.0);
        assert_eq!(anim.value_at(1000), 1.0);
    }

    #[test]
    fn test_animation_with_delay() {
        let anim = Animation::new("opacity", 0.0, 1.0, 1000).delay(500);

        assert_eq!(anim.value_at(0), 0.0);
        assert_eq!(anim.value_at(400), 0.0); // Still in delay
        assert!(anim.value_at(1000) > 0.0); // Started animating
    }

    #[test]
    fn test_transition_css() {
        let transition = Transition::new("opacity", 300);
        let css = transition.to_css();

        assert!(css.contains("opacity"));
        assert!(css.contains("300ms"));
    }

    #[test]
    fn test_spring_physics() {
        let spring = Spring::new();
        let value = spring.value_at(0.0, 100.0, 0.0, 0.1);

        // Should move towards target
        assert!(value > 0.0 && value < 100.0);
    }

    #[test]
    fn test_animated_value() {
        let mut value = AnimatedValue::new(0.0);
        assert_eq!(value.get(), 0.0);

        value.animate_to(100.0, 1000, Easing::Linear);
        value.tick(500);

        // Should be roughly halfway
        assert!(value.get() > 0.0 && value.get() < 100.0);
    }

    #[test]
    fn test_keyframes_css() {
        let mut props_0 = HashMap::new();
        props_0.insert("opacity".to_string(), "0".to_string());

        let mut props_100 = HashMap::new();
        props_100.insert("opacity".to_string(), "1".to_string());

        let keyframes = Keyframes::new("fadeIn")
            .add_frame(0.0, props_0)
            .add_frame(1.0, props_100);

        let css = keyframes.to_css();
        assert!(css.contains("@keyframes fadeIn"));
        assert!(css.contains("0%"));
        assert!(css.contains("100%"));
        assert!(css.contains("opacity"));
    }

    #[test]
    fn test_stagger_animation() {
        let base = Animation::new("opacity", 0.0, 1.0, 300);
        let stagger = StaggerAnimation::new(base, 100);

        let anim_0 = stagger.for_index(0);
        let anim_2 = stagger.for_index(2);

        assert_eq!(anim_0.delay, 0);
        assert_eq!(anim_2.delay, 200);
    }

    #[test]
    fn test_parallax() {
        let parallax = Parallax::new(0.5);
        assert_eq!(parallax.calculate_offset(100.0), 50.0);
        assert_eq!(parallax.calculate_offset(200.0), 100.0);
    }
}
