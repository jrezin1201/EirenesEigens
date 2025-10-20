# raven-ui

Complete UI component library for RavensOne applications.

## Features

- 10+ production-ready components
- Full reactive system with Signals
- Accessibility built-in (ARIA, keyboard navigation)
- Theming system with CSS variables
- Zero runtime dependencies

## Components

- **Button** - Interactive buttons with 6 variants
- **Input** - Form inputs with validation
- **Card** - Content containers
- **Modal** - Dialogs and modals
- **Dropdown** - Select menus
- **Tabs** - Tabbed interfaces
- **Accordion** - Collapsible panels
- **Tooltip** - Hover tooltips
- **Badge** - Status indicators
- **Spinner** - Loading indicators

## Installation

```bash
raven pkg add raven-ui
```

## Usage

```raven
import { Button, Input, Card } from "raven-ui";

component App() {
    let name = Signal::new("");
    
    <Card title="Welcome">
        <Input label="Name" value={name.get()} />
        <Button variant="primary">Submit</Button>
    </Card>
}
```

## License

MIT
