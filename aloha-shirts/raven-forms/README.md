# raven-forms

Powerful form handling and validation library for RavensOne with reactive validation, field management, and form builders.

## Features

- **Reactive Forms**: Automatic validation and state updates
- **Built-in Validators**: Email, URL, min/max, pattern matching, and more
- **Custom Validation**: Easy-to-write custom validators
- **Field-level Validation**: Real-time feedback as users type
- **Form-level Validation**: Cross-field validation rules
- **Async Validation**: Support for server-side validation
- **Form Builders**: Declarative form construction
- **Type Safety**: Full type inference for form values
- **Error Messages**: Customizable error messages
- **Touch Tracking**: Validate only touched fields
- **Dirty State**: Track which fields have been modified

## Installation

```bash
raven pkg add raven-forms
```

## Quick Start

### Basic Form

```raven
import { useForm, required, email } from "raven-forms"

component LoginForm() {
    let form = useForm({
        email: {
            initial: "",
            validators: [required(), email()]
        },
        password: {
            initial: "",
            validators: [required(), minLength(8)]
        }
    });

    let submit = (e) => {
        e.preventDefault();

        if form.is_valid() {
            console.log("Form values:", form.values());
        }
    };

    <form onSubmit={submit}>
        <div>
            <input
                type="email"
                value={form.get("email")}
                onInput={(e) => form.set("email", e.target.value)}
                onBlur={() => form.touch("email")}
            />
            {form.error("email").map(|err| <span class="error">{err}</span>)}
        </div>

        <div>
            <input
                type="password"
                value={form.get("password")}
                onInput={(e) => form.set("password", e.target.value)}
                onBlur={() => form.touch("password")}
            />
            {form.error("password").map(|err| <span class="error">{err}</span>)}
        </div>

        <button type="submit" disabled={!form.is_valid()}>
            Login
        </button>
    </form>
}
```

### Form Components

```raven
import { Form, Field, SubmitButton } from "raven-forms"

component SignupForm() {
    let handle_submit = (values) => {
        console.log("Submitting:", values);
    };

    <Form onSubmit={handle_submit}>
        <Field
            name="username"
            label="Username"
            validators={[required(), minLength(3)]}
        />

        <Field
            name="email"
            label="Email"
            type="email"
            validators={[required(), email()]}
        />

        <Field
            name="password"
            label="Password"
            type="password"
            validators={[required(), minLength(8), hasUppercase()]}
        />

        <Field
            name="confirm_password"
            label="Confirm Password"
            type="password"
            validators={[required(), matchesField("password")]}
        />

        <SubmitButton>Sign Up</SubmitButton>
    </Form>
}
```

## Validators

### Built-in Validators

```raven
import {
    required, email, url, min, max, minLength, maxLength,
    pattern, alphanumeric, numeric, alpha
} from "raven-forms"

// Required field
required()
required("This field is required")

// Email validation
email()
email("Please enter a valid email")

// URL validation
url()

// Numeric constraints
min(5)
max(100)
minLength(3)
maxLength(50)

// Pattern matching
pattern(/^[A-Z0-9]+$/)
pattern(/^\d{5}$/, "Must be 5 digits")

// Character type
alphanumeric()
numeric()
alpha()
```

### Custom Validators

```raven
import { Validator } from "raven-forms"

fn customValidator(value: String) -> Option<String> {
    if value.contains("bad_word") {
        Some("Please remove inappropriate content".to_string())
    } else {
        None
    }
}

// Use in form
<Field
    name="comment"
    validators={[required(), customValidator]}
/>
```

### Async Validators

```raven
import { asyncValidator } from "raven-forms"

let check_username_available = asyncValidator(async (value) => {
    let response = await fetch(`/api/check-username/${value}`);
    let data = await response.json();

    if data.available {
        None
    } else {
        Some("Username is already taken".to_string())
    }
});

<Field
    name="username"
    validators={[required()]}
    asyncValidators={[check_username_available]}
/>
```

## Advanced Usage

### Cross-field Validation

```raven
import { useForm, FormValidator } from "raven-forms"

let passwords_match: FormValidator = (values) => {
    if values.get("password") != values.get("confirm_password") {
        {
            "confirm_password": "Passwords must match"
        }
    } else {
        {}
    }
};

let form = useForm({
    password: { initial: "", validators: [required()] },
    confirm_password: { initial: "", validators: [required()] }
}, [passwords_match]);
```

### Conditional Validation

```raven
let conditional_required = (condition: bool) => {
    move |value| {
        if condition && value.is_empty() {
            Some("This field is required".to_string())
        } else {
            None
        }
    }
};

component ConditionalForm() {
    let show_extra = signal(false);

    <Form>
        <Field name="email" validators={[required(), email()]} />

        <label>
            <input
                type="checkbox"
                checked={show_extra.get()}
                onChange={(e) => show_extra.set(e.target.checked)}
            />
            Show extra field
        </label>

        {if show_extra.get() {
            <Field
                name="phone"
                validators={[conditional_required(show_extra.get())]}
            />
        }}
    </Form>
}
```

### Form Arrays

```raven
import { useFormArray } from "raven-forms"

component DynamicList() {
    let items = useFormArray([
        { name: "", email: "" }
    ]);

    let add_item = () => {
        items.push({ name: "", email: "" });
    };

    let remove_item = (index) => {
        items.remove(index);
    };

    <div>
        {items.fields().enumerate().map(|(i, item)| {
            <div key={i}>
                <Field
                    name={`items.${i}.name`}
                    value={item.name}
                    validators={[required()]}
                />
                <Field
                    name={`items.${i}.email`}
                    value={item.email}
                    validators={[email()]}
                />
                <button onClick={() => remove_item(i)}>Remove</button>
            </div>
        })}

        <button onClick={add_item}>Add Item</button>
    </div>
}
```

### Form Wizard

```raven
import { useFormWizard } from "raven-forms"

component MultiStepForm() {
    let wizard = useFormWizard({
        steps: [
            {
                name: "personal",
                fields: {
                    first_name: { initial: "", validators: [required()] },
                    last_name: { initial: "", validators: [required()] }
                }
            },
            {
                name: "contact",
                fields: {
                    email: { initial: "", validators: [required(), email()] },
                    phone: { initial: "", validators: [required()] }
                }
            },
            {
                name: "preferences",
                fields: {
                    newsletter: { initial: false, validators: [] }
                }
            }
        ]
    });

    <div>
        <div class="progress">
            Step {wizard.current_step() + 1} of {wizard.total_steps()}
        </div>

        {wizard.render_step()}

        <div class="buttons">
            {if wizard.can_go_back() {
                <button onClick={wizard.previous}>Back</button>
            }}

            {if wizard.can_go_next() {
                <button onClick={wizard.next}>Next</button>
            } else {
                <button onClick={wizard.submit}>Submit</button>
            }}
        </div>
    </div>
}
```

## API Reference

### `useForm(config, validators?)`

Create a new form instance.

**Parameters**:
- `config`: Object mapping field names to field configuration
- `validators`: Optional array of form-level validators

**Returns**: Form instance

### Form Methods

- `get(field)` - Get field value
- `set(field, value)` - Set field value
- `touch(field)` - Mark field as touched
- `error(field)` - Get field error message
- `is_valid()` - Check if entire form is valid
- `is_touched(field)` - Check if field is touched
- `is_dirty(field)` - Check if field is modified
- `values()` - Get all form values
- `errors()` - Get all error messages
- `reset()` - Reset form to initial state
- `submit()` - Submit the form

### Field Configuration

```raven
{
    initial: any,           // Initial value
    validators: Vec<Validator>, // Sync validators
    asyncValidators: Vec<AsyncValidator>, // Async validators
    validateOn: "change" | "blur" | "submit" // When to validate
}
```

## Best Practices

1. **Validate on blur**: Use `validateOn: "blur"` for better UX
2. **Custom error messages**: Always provide user-friendly messages
3. **Show errors after touch**: Only show errors for touched fields
4. **Disable submit**: Disable submit button when form is invalid
5. **Loading states**: Show loading indicator during async validation
6. **Reset after submit**: Reset form after successful submission

## Examples

See `/examples` for complete examples:
- `login-form.raven` - Simple login form
- `signup-form.raven` - Registration with validation
- `wizard-form.raven` - Multi-step form wizard
- `dynamic-fields.raven` - Form with dynamic field arrays
- `async-validation.raven` - Server-side validation

## Performance

raven-forms uses fine-grained reactivity for optimal performance:
- Only validates changed fields
- Debounces async validation
- Memoizes validation results
- Minimal re-renders

## License

MIT License - See LICENSE file for details
