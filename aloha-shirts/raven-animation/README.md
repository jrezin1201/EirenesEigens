# raven-animation

Advanced animation library for RavensOne featuring spring physics, timeline animations, gestures, and smooth transitions.

## Features

- **Spring Physics**: Natural, physics-based animations
- **Keyframe Animations**: CSS-like keyframe definitions
- **Timeline Control**: Orchestrate complex animation sequences
- **Easing Functions**: 30+ built-in easing curves
- **Gesture Recognition**: Drag, swipe, pinch, and rotate gestures
- **Transition Components**: Drop-in animated transitions
- **Auto-animation**: Automatic layout animations
- **Performance Optimized**: GPU-accelerated when possible

## Installation

```bash
raven pkg add raven-animation
```

## Quick Start

```raven
import { animate, useSpring } from "raven-animation"

component Box() {
    let spring = useSpring(0, {
        stiffness: 200,
        damping: 20
    });

    <div
        style={`transform: translateY(${spring.get()}px)`}
        onClick={() => spring.set(spring.get() == 0 ? 200 : 0)}
    >
        Bouncy!
    </div>
}
```

## License

MIT
