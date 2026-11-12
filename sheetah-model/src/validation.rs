use loro::{LoroDoc, LoroMap, LoroList, LoroValue};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

impl From<&str> for ValidationSeverity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "warning" => ValidationSeverity::Warning,
            "info" => ValidationSeverity::Info,
            _ => ValidationSeverity::Error,
        }
    }
}

impl ValidationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationSeverity::Error => "error",
            ValidationSeverity::Warning => "warning",
            ValidationSeverity::Info => "info",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationViolation {
    pub path: String,
    pub message: String,
    pub code: String,
    pub severity: ValidationSeverity,
}

/// A trait for components that can validate a Loro document.
pub trait Validator: Send + Sync {
    fn name(&self) -> &'static str;
    /// Validates the document and returns a list of violations.
    fn validate(&self, doc: &LoroDoc) -> Vec<ValidationViolation>;
}

pub struct ValidationManager {
    validators: Vec<Box<dyn Validator>>,
    error_key: &'static str,
}

impl ValidationManager {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            error_key: "_validation_errors",
        }
    }

    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }

    /// Runs all validators and updates the document's `_validation_errors` list.
    /// This should be called by the Sync Engine after applying a change.
    pub fn validate_and_apply(&self, doc: &mut LoroDoc) -> Result<(), Box<dyn std::error::Error>> {
        let mut all_violations = Vec::new();
        for validator in &self.validators {
            all_violations.extend(validator.validate(doc));
        }

        let errors_list = doc.get_list(self.error_key);
        
        // For simplicity in this prototype, we clear and replace.
        // In a high-perf production system, we might want to diff and patch.
        errors_list.clear();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for violation in all_violations {
            let mut error_map = LoroMap::new();
            error_map.insert("path", violation.path)?;
            error_map.insert("message", violation.message)?;
            error_map.insert("code", violation.code)?;
            error_map.insert("level", violation.severity.as_str().to_string())?;
            error_map.insert("ts", now as i64)?;
            
            errors_list.insert(errors_list.len(), error_map.into())?;
        }

        Ok(())
    }
}
