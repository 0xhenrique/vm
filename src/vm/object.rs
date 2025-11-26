// Heap-allocated objects
// This module will contain heap object representations
// and provide hooks for future GC integration

use super::value::Value;

// Placeholder for heap objects
// In the future, this will contain object representations
// that support garbage collection

pub struct HeapObject {
    pub value: Value,
}
