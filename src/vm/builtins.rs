// Built-in function registration and management
// This will contain helpers for registering built-in functions

use super::value::Value;
use super::errors::RuntimeError;

// Placeholder for future built-in function registry
pub type BuiltinFn = fn(&[Value]) -> Result<Value, RuntimeError>;
