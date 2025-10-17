# raven-i18n

Complete internationalization (i18n) and localization (l10n) library for RavensOne applications with translation management, pluralization, and formatting.

## Features

- **Translation Management**: Easy-to-use translation system with nested keys
- **Pluralization**: Automatic plural form selection for all languages
- **Number Formatting**: Locale-aware number, currency, and percentage formatting
- **Date/Time Formatting**: Localized date and time display
- **Dynamic Loading**: Lazy-load translations for better performance
- **Fallback Locales**: Graceful fallback to default language
- **Interpolation**: Variable substitution in translations
- **Namespaces**: Organize translations by feature or domain
- **React to Language Changes**: Automatic UI updates when language changes
- **RTL Support**: Right-to-left language support

## Installation

```bash
raven pkg add raven-i18n
```

## Quick Start

### Initialize i18n

```raven
import { create_i18n } from "raven-i18n"

let i18n = create_i18n({
    default_locale: "en",
    fallback_locale: "en",
    translations: {
        en: {
            welcome: "Welcome",
            greeting: "Hello, {name}!",
            items: {
                one: "{count} item",
                other: "{count} items"
            }
        },
        es: {
            welcome: "Bienvenido",
            greeting: "¡Hola, {name}!",
            items: {
                one: "{count} artículo",
                other: "{count} artículos"
            }
        }
    }
});
```

### Basic Translation

```raven
// Get translation
let message = i18n.t("welcome"); // "Welcome"

// With interpolation
let greeting = i18n.t("greeting", { name: "Alice" }); // "Hello, Alice!"

// Pluralization
let count_message = i18n.t("items", { count: 1 }); // "1 item"
let count_message2 = i18n.t("items", { count: 5 }); // "5 items"
```

### Change Language

```raven
i18n.set_locale("es");

let message = i18n.t("welcome"); // "Bienvenido"
let greeting = i18n.t("greeting", { name: "Alice" }); // "¡Hola, Alice!"
```

### Use in Components

```raven
import { useTranslation } from "raven-i18n"

component Welcome() {
    let t = useTranslation();

    <div>
        <h1>{t("welcome")}</h1>
        <p>{t("greeting", { name: "World" })}</p>

        <button onClick={() => i18n.set_locale("es")}>
            Español
        </button>
        <button onClick={() => i18n.set_locale("en")}>
            English
        </button>
    </div>
}
```

## Advanced Features

### Namespaces

Organize translations by feature:

```raven
let i18n = create_i18n({
    default_locale: "en",
    namespaces: ["common", "auth", "dashboard"],
    translations: {
        en: {
            common: {
                save: "Save",
                cancel: "Cancel"
            },
            auth: {
                login: "Login",
                logout: "Logout"
            },
            dashboard: {
                title: "Dashboard",
                stats: "Statistics"
            }
        }
    }
});

// Use namespace
let login_text = i18n.t("auth:login"); // "Login"
let save_text = i18n.t("common:save"); // "Save"
```

### Lazy Loading

Load translations on demand:

```raven
import { create_i18n, load_locale } from "raven-i18n"

let i18n = create_i18n({
    default_locale: "en",
    lazy: true
});

// Load locale when needed
await load_locale("fr", "/locales/fr.json");

i18n.set_locale("fr");
```

### Number Formatting

```raven
import { formatNumber, formatCurrency, formatPercent } from "raven-i18n"

// Number formatting
formatNumber(1234.56, "en-US"); // "1,234.56"
formatNumber(1234.56, "de-DE"); // "1.234,56"

// Currency formatting
formatCurrency(1234.56, "USD", "en-US"); // "$1,234.56"
formatCurrency(1234.56, "EUR", "de-DE"); // "1.234,56 €"

// Percentage formatting
formatPercent(0.1234, "en-US"); // "12.34%"
```

### Date/Time Formatting

```raven
import { formatDate, formatTime, formatRelative } from "raven-i18n"

let date = Date::now();

// Date formatting
formatDate(date, "en-US"); // "12/25/2025"
formatDate(date, "en-GB"); // "25/12/2025"
formatDate(date, "de-DE"); // "25.12.2025"

// Time formatting
formatTime(date, "en-US"); // "3:45 PM"
formatTime(date, "en-GB"); // "15:45"

// Relative time
formatRelative(date, "en"); // "2 hours ago"
```

### Pluralization Rules

Automatic plural form selection:

```raven
let i18n = create_i18n({
    default_locale: "en",
    translations: {
        en: {
            apples: {
                zero: "No apples",
                one: "One apple",
                other: "{count} apples"
            }
        },
        pl: {  // Polish has more plural forms
            apples: {
                one: "{count} jabłko",
                few: "{count} jabłka",
                many: "{count} jabłek",
                other: "{count} jabłek"
            }
        }
    }
});

i18n.t("apples", { count: 0 }); // "No apples"
i18n.t("apples", { count: 1 }); // "One apple"
i18n.t("apples", { count: 5 }); // "5 apples"
```

### Context-based Translation

```raven
let i18n = create_i18n({
    default_locale: "en",
    translations: {
        en: {
            friend: {
                male: "He is my friend",
                female: "She is my friend"
            }
        }
    }
});

i18n.t("friend", { context: "male" }); // "He is my friend"
i18n.t("friend", { context: "female" }); // "She is my friend"
```

### RTL Support

```raven
import { is_rtl, get_text_direction } from "raven-i18n"

i18n.set_locale("ar"); // Arabic

is_rtl("ar"); // true
get_text_direction("ar"); // "rtl"

// In component
component App() {
    let dir = get_text_direction(i18n.get_locale());

    <div dir={dir} class={if dir == "rtl" { "rtl-layout" } else { "" }}>
        <h1>{i18n.t("welcome")}</h1>
    </div>
}
```

## File Structure

### JSON Translation Files

```json
{
  "common": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete"
  },
  "auth": {
    "login": "Login",
    "logout": "Logout",
    "forgot_password": "Forgot password?"
  },
  "validation": {
    "required": "This field is required",
    "email": "Please enter a valid email",
    "min_length": "Must be at least {min} characters"
  }
}
```

### YAML Translation Files

```yaml
common:
  save: Save
  cancel: Cancel
  delete: Delete

auth:
  login: Login
  logout: Logout
  forgot_password: Forgot password?

validation:
  required: This field is required
  email: Please enter a valid email
  min_length: Must be at least {min} characters
```

## API Reference

### `create_i18n(config)`

Create an i18n instance.

**Parameters**:
- `default_locale`: Default language code
- `fallback_locale`: Fallback language
- `translations`: Translation data
- `namespaces`: Optional namespaces
- `lazy`: Enable lazy loading

### `i18n.t(key, params?)`

Translate a key with optional parameters.

### `i18n.set_locale(locale)`

Change the current language.

### `i18n.get_locale()`

Get the current language.

### `i18n.add_translations(locale, translations)`

Add translations dynamically.

### `useTranslation()`

Hook for using translations in components.

## Best Practices

1. **Organize by feature**: Use namespaces to group related translations
2. **Provide fallbacks**: Always include fallback translations
3. **Use interpolation**: Keep dynamic content in variables
4. **Lazy load**: Load translations on demand for better performance
5. **Test all languages**: Verify translations in all supported languages
6. **Use ICU format**: For complex pluralization and formatting
7. **RTL testing**: Test RTL layouts thoroughly

## Examples

See `/examples` for complete examples:
- `basic-i18n.raven` - Simple translation example
- `language-switcher.raven` - Language selector component
- `pluralization.raven` - Plural forms example
- `lazy-loading.raven` - Dynamic translation loading
- `rtl-layout.raven` - Right-to-left layout example

## Supported Locales

The library supports all standard locale codes (e.g., en, en-US, es, es-MX, fr, de, ar, zh, ja, etc.).

## License

MIT License - See LICENSE file for details
