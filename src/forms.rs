// Forms & Validation System for RavensOne
// Form state management, field validation, error handling, and submission

use crate::reactive::{Signal, Computed};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Validation rule type
pub type ValidationRule<T> = Box<dyn Fn(&T) -> Result<(), String>>;

/// Field state for a single form field
#[derive(Clone)]
pub struct Field<T: Clone> {
    pub value: Signal<T>,
    pub error: Signal<Option<String>>,
    pub touched: Signal<bool>,
    pub dirty: Signal<bool>,
    validators: Rc<RefCell<Vec<ValidationRule<T>>>>,
}

impl<T: Clone + 'static> Field<T> {
    pub fn new(initial_value: T) -> Self {
        Field {
            value: Signal::new(initial_value),
            error: Signal::new(None),
            touched: Signal::new(false),
            dirty: Signal::new(false),
            validators: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Add a validation rule
    pub fn add_validator<F>(&self, validator: F)
    where
        F: Fn(&T) -> Result<(), String> + 'static,
    {
        self.validators.borrow_mut().push(Box::new(validator));
    }

    /// Validate the field
    pub fn validate(&self) -> bool {
        let value = self.value.get();

        for validator in self.validators.borrow().iter() {
            match validator(&value) {
                Ok(_) => continue,
                Err(msg) => {
                    self.error.set(Some(msg));
                    return false;
                }
            }
        }

        self.error.set(None);
        true
    }

    /// Set value and mark as dirty
    pub fn set_value(&self, new_value: T) {
        self.value.set(new_value);
        self.dirty.set(true);
    }

    /// Mark field as touched
    pub fn mark_touched(&self) {
        self.touched.set(true);
    }

    /// Reset field to initial state
    pub fn reset(&self, initial_value: T) {
        self.value.set(initial_value);
        self.error.set(None);
        self.touched.set(false);
        self.dirty.set(false);
    }

    /// Check if field is valid
    pub fn is_valid(&self) -> bool {
        self.error.get().is_none()
    }

    /// Check if field should show error (touched and has error)
    pub fn should_show_error(&self) -> bool {
        self.touched.get() && self.error.get().is_some()
    }
}

/// Form state manager
pub struct Form {
    fields: Rc<RefCell<HashMap<String, Box<dyn FieldValidator>>>>,
    pub submitting: Signal<bool>,
    pub submit_count: Signal<u32>,
    pub is_valid: Computed<bool>,
}

trait FieldValidator {
    fn validate(&self) -> bool;
    fn mark_touched(&self);
    fn is_valid(&self) -> bool;
}

impl<T: Clone + 'static> FieldValidator for Field<T> {
    fn validate(&self) -> bool {
        Field::validate(self)
    }

    fn mark_touched(&self) {
        Field::mark_touched(self)
    }

    fn is_valid(&self) -> bool {
        Field::is_valid(self)
    }
}

impl Form {
    pub fn new() -> Self {
        let fields: HashMap<String, Box<dyn FieldValidator>> = HashMap::new();
        let fields_rc = Rc::new(RefCell::new(fields));
        let fields_clone = fields_rc.clone();

        let is_valid = Computed::new(move || {
            fields_clone.borrow().values().all(|field| field.is_valid())
        });

        Form {
            fields: fields_rc,
            submitting: Signal::new(false),
            submit_count: Signal::new(0),
            is_valid,
        }
    }

    /// Register a field with the form
    pub fn register_field<T: Clone + 'static>(&mut self, name: &str, field: Field<T>) {
        self.fields.borrow_mut().insert(name.to_string(), Box::new(field));
    }

    /// Validate all fields
    pub fn validate_all(&self) -> bool {
        let mut all_valid = true;

        for field in self.fields.borrow().values() {
            if !field.validate() {
                all_valid = false;
            }
        }

        all_valid
    }

    /// Mark all fields as touched
    pub fn mark_all_touched(&self) {
        for field in self.fields.borrow().values() {
            field.mark_touched();
        }
    }

    /// Submit the form
    pub fn submit<F>(&self, handler: F)
    where
        F: FnOnce() -> (),
    {
        // Mark all fields as touched
        self.mark_all_touched();

        // Validate all fields
        if !self.validate_all() {
            println!("[Form] Validation failed");
            return;
        }

        // Set submitting state
        self.submitting.set(true);
        self.submit_count.set(self.submit_count.get() + 1);

        // Call the submit handler
        handler();

        // Reset submitting state
        self.submitting.set(false);
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

/// Common validators
pub mod validators {
    use super::*;

    /// Required field validator
    pub fn required() -> impl Fn(&String) -> Result<(), String> {
        |value: &String| {
            if value.trim().is_empty() {
                Err("This field is required".to_string())
            } else {
                Ok(())
            }
        }
    }

    /// Minimum length validator
    pub fn min_length(min: usize) -> impl Fn(&String) -> Result<(), String> {
        move |value: &String| {
            if value.len() < min {
                Err(format!("Must be at least {} characters", min))
            } else {
                Ok(())
            }
        }
    }

    /// Maximum length validator
    pub fn max_length(max: usize) -> impl Fn(&String) -> Result<(), String> {
        move |value: &String| {
            if value.len() > max {
                Err(format!("Must be at most {} characters", max))
            } else {
                Ok(())
            }
        }
    }

    /// Email validator
    pub fn email() -> impl Fn(&String) -> Result<(), String> {
        |value: &String| {
            if value.contains('@') && value.contains('.') {
                Ok(())
            } else {
                Err("Invalid email address".to_string())
            }
        }
    }

    /// Pattern validator (basic regex-like)
    pub fn pattern(pattern: &'static str, message: &'static str) -> impl Fn(&String) -> Result<(), String> {
        move |value: &String| {
            // Simple pattern matching (not full regex)
            if pattern == "numeric" && value.chars().all(|c| c.is_numeric()) {
                Ok(())
            } else if pattern == "alphanumeric" && value.chars().all(|c| c.is_alphanumeric()) {
                Ok(())
            } else if pattern == "alpha" && value.chars().all(|c| c.is_alphabetic()) {
                Ok(())
            } else {
                Err(message.to_string())
            }
        }
    }

    /// Minimum value validator for numbers
    pub fn min_value(min: i32) -> impl Fn(&i32) -> Result<(), String> {
        move |value: &i32| {
            if *value < min {
                Err(format!("Must be at least {}", min))
            } else {
                Ok(())
            }
        }
    }

    /// Maximum value validator for numbers
    pub fn max_value(max: i32) -> impl Fn(&i32) -> Result<(), String> {
        move |value: &i32| {
            if *value > max {
                Err(format!("Must be at most {}", max))
            } else {
                Ok(())
            }
        }
    }

    /// Range validator for numbers
    pub fn range(min: i32, max: i32) -> impl Fn(&i32) -> Result<(), String> {
        move |value: &i32| {
            if *value < min || *value > max {
                Err(format!("Must be between {} and {}", min, max))
            } else {
                Ok(())
            }
        }
    }

    /// Custom validator
    pub fn custom<T, F>(validator: F, message: &'static str) -> impl Fn(&T) -> Result<(), String>
    where
        F: Fn(&T) -> bool + 'static,
    {
        move |value: &T| {
            if validator(value) {
                Ok(())
            } else {
                Err(message.to_string())
            }
        }
    }

    /// Matches another field (for password confirmation)
    pub fn matches(other_value: String) -> impl Fn(&String) -> Result<(), String> {
        move |value: &String| {
            if *value == other_value {
                Ok(())
            } else {
                Err("Fields do not match".to_string())
            }
        }
    }
}

/// Form builder for creating forms with validation
pub struct FormBuilder {
    form: Form,
}

impl FormBuilder {
    pub fn new() -> Self {
        FormBuilder {
            form: Form::new(),
        }
    }

    /// Add a text field with validators
    pub fn text_field(
        mut self,
        name: &str,
        initial_value: &str,
        validators: Vec<Box<dyn Fn(&String) -> Result<(), String>>>,
    ) -> Self {
        let field = Field::new(initial_value.to_string());

        for validator in validators {
            field.add_validator(validator);
        }

        self.form.register_field(name, field);
        self
    }

    /// Add a number field with validators
    pub fn number_field(
        mut self,
        name: &str,
        initial_value: i32,
        validators: Vec<Box<dyn Fn(&i32) -> Result<(), String>>>,
    ) -> Self {
        let field = Field::new(initial_value);

        for validator in validators {
            field.add_validator(validator);
        }

        self.form.register_field(name, field);
        self
    }

    /// Build the form
    pub fn build(self) -> Form {
        self.form
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Form field component helper
#[derive(Clone)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

#[derive(Clone)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    TextArea,
    Select(Vec<String>),
    Checkbox,
    Radio(Vec<String>),
}

impl FormField {
    pub fn new(name: &str, label: &str, field_type: FieldType) -> Self {
        FormField {
            name: name.to_string(),
            label: label.to_string(),
            field_type,
            placeholder: None,
            help_text: None,
        }
    }

    pub fn placeholder(mut self, text: &str) -> Self {
        self.placeholder = Some(text.to_string());
        self
    }

    pub fn help(mut self, text: &str) -> Self {
        self.help_text = Some(text.to_string());
        self
    }
}

/// Async form submission handler
pub struct AsyncFormSubmit<T: Clone> {
    pub loading: Signal<bool>,
    pub error: Signal<Option<String>>,
    pub data: Signal<Option<T>>,
}

impl<T: Clone> AsyncFormSubmit<T> {
    pub fn new() -> Self {
        AsyncFormSubmit {
            loading: Signal::new(false),
            error: Signal::new(None),
            data: Signal::new(None),
        }
    }

    /// Submit form asynchronously (simulated)
    pub fn submit<F>(&self, handler: F)
    where
        F: FnOnce() -> Result<T, String>,
    {
        self.loading.set(true);
        self.error.set(None);

        match handler() {
            Ok(result) => {
                self.data.set(Some(result));
                self.error.set(None);
            }
            Err(err) => {
                self.error.set(Some(err));
            }
        }

        self.loading.set(false);
    }

    /// Reset submission state
    pub fn reset(&self) {
        self.loading.set(false);
        self.error.set(None);
        self.data.set(None);
    }
}

impl<T: Clone> Default for AsyncFormSubmit<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Form array for dynamic form fields
pub struct FormArray<T: Clone> {
    pub fields: Signal<Vec<Field<T>>>,
}

impl<T: Clone + Default + 'static> FormArray<T> {
    pub fn new() -> Self {
        FormArray {
            fields: Signal::new(Vec::new()),
        }
    }

    /// Add a new field
    pub fn push(&self, field: Field<T>) {
        let mut fields = self.fields.get();
        fields.push(field);
        self.fields.set(fields);
    }

    /// Remove a field by index
    pub fn remove(&self, index: usize) {
        let mut fields = self.fields.get();
        if index < fields.len() {
            fields.remove(index);
            self.fields.set(fields);
        }
    }

    /// Get field count
    pub fn len(&self) -> usize {
        self.fields.get().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fields.get().is_empty()
    }

    /// Validate all fields
    pub fn validate_all(&self) -> bool {
        self.fields.get().iter().all(|field| field.validate())
    }
}

impl<T: Clone + Default + 'static> Default for FormArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Debounced validation
pub struct DebouncedValidator {
    pub timeout: u32,
    pub pending: Signal<bool>,
}

impl DebouncedValidator {
    pub fn new(timeout_ms: u32) -> Self {
        DebouncedValidator {
            timeout: timeout_ms,
            pending: Signal::new(false),
        }
    }

    /// Validate with debounce (simulated)
    pub fn validate<F>(&self, validator: F)
    where
        F: Fn() -> bool,
    {
        self.pending.set(true);

        // In a real implementation, this would use setTimeout
        // For now, just validate immediately
        let _is_valid = validator();

        self.pending.set(false);
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, AsyncFormSubmit, FormArray, validators};

    #[test]
    fn test_field_creation() {
        let field = Field::new(String::from("test"));
        assert_eq!(field.value.get(), "test");
        assert_eq!(field.error.get(), None);
        assert!(!field.touched.get());
        assert!(!field.dirty.get());
    }

    #[test]
    fn test_required_validator() {
        let field = Field::new(String::new());
        field.add_validator(validators::required());

        assert!(!field.validate());
        assert!(field.error.get().is_some());

        field.set_value(String::from("test"));
        assert!(field.validate());
        assert!(field.error.get().is_none());
    }

    #[test]
    fn test_min_length_validator() {
        let field = Field::new(String::from("ab"));
        field.add_validator(validators::min_length(3));

        assert!(!field.validate());

        field.set_value(String::from("abc"));
        assert!(field.validate());
    }

    #[test]
    fn test_email_validator() {
        let field = Field::new(String::from("invalid"));
        field.add_validator(validators::email());

        assert!(!field.validate());

        field.set_value(String::from("test@example.com"));
        assert!(field.validate());
    }

    #[test]
    fn test_number_range_validator() {
        let field = Field::new(5);
        field.add_validator(validators::range(1, 10));

        assert!(field.validate());

        field.set_value(15);
        assert!(!field.validate());
    }

    #[test]
    fn test_field_touched_state() {
        let field = Field::new(String::new());

        assert!(!field.touched.get());
        field.mark_touched();
        assert!(field.touched.get());
    }

    #[test]
    fn test_field_dirty_state() {
        let field = Field::new(String::from("initial"));

        assert!(!field.dirty.get());
        field.set_value(String::from("changed"));
        assert!(field.dirty.get());
    }

    #[test]
    fn test_field_reset() {
        let field = Field::new(String::from("initial"));
        field.set_value(String::from("changed"));
        field.mark_touched();
        field.add_validator(validators::required());
        field.validate();

        field.reset(String::from("reset"));

        assert_eq!(field.value.get(), "reset");
        assert!(!field.touched.get());
        assert!(!field.dirty.get());
    }

    #[test]
    fn test_multiple_validators() {
        let field = Field::new(String::from("ab"));
        field.add_validator(validators::required());
        field.add_validator(validators::min_length(3));

        assert!(!field.validate());

        field.set_value(String::from("abc"));
        assert!(field.validate());
    }

    #[test]
    fn test_async_form_submit() {
        let submit = AsyncFormSubmit::<String>::new();

        assert!(!submit.loading.get());
        assert!(submit.data.get().is_none());

        submit.submit(|| Ok(String::from("success")));

        assert!(!submit.loading.get());
        assert_eq!(submit.data.get(), Some(String::from("success")));
    }

    #[test]
    fn test_form_array() {
        let array = FormArray::<String>::new();

        assert_eq!(array.len(), 0);
        assert!(array.is_empty());

        array.push(Field::new(String::from("item1")));
        array.push(Field::new(String::from("item2")));

        assert_eq!(array.len(), 2);
        assert!(!array.is_empty());

        array.remove(0);
        assert_eq!(array.len(), 1);
    }
}
