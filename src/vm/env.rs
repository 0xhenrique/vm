// Environment and global state management
// This will contain global variables, symbol tables, etc.

use std::collections::HashMap;
use super::value::Value;

pub type GlobalEnv = HashMap<String, Value>;
