// FFI (Foreign Function Interface) subsystem
// Allows Lisp code to call C functions from shared libraries

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_void};
use std::sync::Arc;

use libloading::{Library, Symbol};
use libffi::middle::{Arg, Builder, CodePtr, Type as FfiTypeNative};

use super::value::Value;
use super::instructions::FfiType;
use super::errors::RuntimeError;

/// Manages loaded libraries and their lifetimes
pub struct FfiState {
    /// Map of library handle (as integer) to loaded Library
    libraries: HashMap<i64, Arc<Library>>,
    /// Counter for generating unique library handles
    next_handle: i64,
    /// Allocated strings that need to be freed (address -> CString)
    allocated_strings: HashMap<i64, CString>,
    /// Allocated memory blocks that need to be freed
    allocated_memory: HashMap<i64, Vec<u8>>,
}

impl FfiState {
    pub fn new() -> Self {
        FfiState {
            libraries: HashMap::new(),
            next_handle: 1, // Start at 1, 0 means "failed to load"
            allocated_strings: HashMap::new(),
            allocated_memory: HashMap::new(),
        }
    }

    /// Load a shared library and return a handle
    pub fn load_library(&mut self, path: &str) -> Result<i64, String> {
        // Try to load the library
        let library = unsafe {
            Library::new(path).map_err(|e| format!("Failed to load library '{}': {}", path, e))?
        };

        let handle = self.next_handle;
        self.next_handle += 1;
        self.libraries.insert(handle, Arc::new(library));
        Ok(handle)
    }

    /// Get a symbol (function pointer) from a loaded library
    pub fn get_symbol(&self, handle: i64, name: &str) -> Result<i64, String> {
        let library = self.libraries.get(&handle)
            .ok_or_else(|| format!("Invalid library handle: {}", handle))?;

        // Get the symbol as a raw pointer
        unsafe {
            let symbol: Symbol<*const ()> = library.get(name.as_bytes())
                .map_err(|e| format!("Failed to find symbol '{}': {}", name, e))?;
            let ptr = *symbol as i64;
            Ok(ptr)
        }
    }

    /// Allocate a C string from a Lisp string, returning the pointer
    pub fn allocate_string(&mut self, s: &str) -> Result<i64, String> {
        let cstring = CString::new(s)
            .map_err(|_| "String contains null bytes".to_string())?;
        let ptr = cstring.as_ptr() as i64;
        self.allocated_strings.insert(ptr, cstring);
        Ok(ptr)
    }

    /// Free an allocated C string
    pub fn free_string(&mut self, ptr: i64) -> bool {
        self.allocated_strings.remove(&ptr).is_some()
    }

    /// Read a C string from a pointer
    pub fn pointer_to_string(&self, ptr: i64) -> Result<String, String> {
        if ptr == 0 {
            return Err("Cannot read string from null pointer".to_string());
        }
        unsafe {
            let cstr = CStr::from_ptr(ptr as *const i8);
            cstr.to_str()
                .map(|s| s.to_string())
                .map_err(|_| "Invalid UTF-8 in C string".to_string())
        }
    }

    /// Allocate raw memory
    pub fn allocate(&mut self, size: usize) -> i64 {
        let mut buffer = vec![0u8; size];
        let ptr = buffer.as_mut_ptr() as i64;
        self.allocated_memory.insert(ptr, buffer);
        ptr
    }

    /// Free allocated memory
    pub fn free(&mut self, ptr: i64) -> bool {
        self.allocated_memory.remove(&ptr).is_some()
    }

    /// Read an i64 from a pointer
    pub fn read_int(&self, ptr: i64) -> Result<i64, String> {
        if ptr == 0 {
            return Err("Cannot read from null pointer".to_string());
        }
        unsafe {
            Ok(*(ptr as *const i64))
        }
    }

    /// Write an i64 to a pointer
    pub fn write_int(&self, ptr: i64, value: i64) -> Result<(), String> {
        if ptr == 0 {
            return Err("Cannot write to null pointer".to_string());
        }
        unsafe {
            *(ptr as *mut i64) = value;
            Ok(())
        }
    }

    /// Read an f64 from a pointer
    pub fn read_float(&self, ptr: i64) -> Result<f64, String> {
        if ptr == 0 {
            return Err("Cannot read from null pointer".to_string());
        }
        unsafe {
            Ok(*(ptr as *const f64))
        }
    }

    /// Write an f64 to a pointer
    pub fn write_float(&self, ptr: i64, value: f64) -> Result<(), String> {
        if ptr == 0 {
            return Err("Cannot write to null pointer".to_string());
        }
        unsafe {
            *(ptr as *mut f64) = value;
            Ok(())
        }
    }

    /// Read a byte from a pointer
    pub fn read_byte(&self, ptr: i64) -> Result<u8, String> {
        if ptr == 0 {
            return Err("Cannot read from null pointer".to_string());
        }
        unsafe {
            Ok(*(ptr as *const u8))
        }
    }

    /// Write a byte to a pointer
    pub fn write_byte(&self, ptr: i64, value: u8) -> Result<(), String> {
        if ptr == 0 {
            return Err("Cannot write to null pointer".to_string());
        }
        unsafe {
            *(ptr as *mut u8) = value;
            Ok(())
        }
    }

    /// Call a foreign function with the given signature
    pub fn call_function(
        &mut self,
        func_ptr: i64,
        args: Vec<Value>,
        arg_types: &[FfiType],
        return_type: &FfiType,
    ) -> Result<Value, RuntimeError> {
        if func_ptr == 0 {
            return Err(RuntimeError::new("Cannot call null function pointer".to_string()));
        }

        if args.len() != arg_types.len() {
            return Err(RuntimeError::new(format!(
                "FFI call: expected {} arguments, got {}",
                arg_types.len(),
                args.len()
            )));
        }

        // Convert arg types to libffi types and prepare argument storage
        let native_arg_types: Vec<FfiTypeNative> = arg_types.iter()
            .map(ffi_type_to_native)
            .collect();
        let native_return_type = ffi_type_to_native(return_type);

        // Build the CIF (Call Interface)
        let cif = Builder::new()
            .args(native_arg_types.iter().cloned())
            .res(native_return_type)
            .into_cif();

        // Prepare argument values - we need to keep ownership of converted values
        let mut int8_args: Vec<i8> = Vec::new();
        let mut int16_args: Vec<i16> = Vec::new();
        let mut int32_args: Vec<i32> = Vec::new();
        let mut int64_args: Vec<i64> = Vec::new();
        let mut uint8_args: Vec<u8> = Vec::new();
        let mut uint16_args: Vec<u16> = Vec::new();
        let mut uint32_args: Vec<u32> = Vec::new();
        let mut uint64_args: Vec<u64> = Vec::new();
        let mut float_args: Vec<f32> = Vec::new();
        let mut double_args: Vec<f64> = Vec::new();
        let mut ptr_args: Vec<*const ()> = Vec::new();
        let mut cstring_args: Vec<CString> = Vec::new();

        // First pass: convert all values and store them
        for (arg, arg_type) in args.iter().zip(arg_types.iter()) {
            match arg_type {
                FfiType::Int8 => {
                    let v = self.value_to_int(arg)?;
                    int8_args.push(v as i8);
                }
                FfiType::Int16 => {
                    let v = self.value_to_int(arg)?;
                    int16_args.push(v as i16);
                }
                FfiType::Int32 => {
                    let v = self.value_to_int(arg)?;
                    int32_args.push(v as i32);
                }
                FfiType::Int64 => {
                    let v = self.value_to_int(arg)?;
                    int64_args.push(v);
                }
                FfiType::UInt8 => {
                    let v = self.value_to_int(arg)?;
                    uint8_args.push(v as u8);
                }
                FfiType::UInt16 => {
                    let v = self.value_to_int(arg)?;
                    uint16_args.push(v as u16);
                }
                FfiType::UInt32 => {
                    let v = self.value_to_int(arg)?;
                    uint32_args.push(v as u32);
                }
                FfiType::UInt64 => {
                    let v = self.value_to_int(arg)?;
                    uint64_args.push(v as u64);
                }
                FfiType::Float => {
                    let v = self.value_to_float(arg)?;
                    float_args.push(v as f32);
                }
                FfiType::Double => {
                    let v = self.value_to_float(arg)?;
                    double_args.push(v);
                }
                FfiType::Pointer => {
                    let v = self.value_to_pointer(arg)?;
                    ptr_args.push(v as *const ());
                }
                FfiType::String => {
                    let s = self.value_to_string(arg)?;
                    let cstring = CString::new(s).map_err(|_|
                        RuntimeError::new("String contains null bytes".to_string()))?;
                    cstring_args.push(cstring);
                }
                FfiType::Void => {
                    return Err(RuntimeError::new("Void type not allowed as argument".to_string()));
                }
            }
        }

        // Second pass: build Arg references
        let mut int8_idx = 0;
        let mut int16_idx = 0;
        let mut int32_idx = 0;
        let mut int64_idx = 0;
        let mut uint8_idx = 0;
        let mut uint16_idx = 0;
        let mut uint32_idx = 0;
        let mut uint64_idx = 0;
        let mut float_idx = 0;
        let mut double_idx = 0;
        let mut ptr_idx = 0;
        let mut cstring_idx = 0;

        let ffi_args: Vec<Arg> = arg_types.iter().map(|arg_type| {
            match arg_type {
                FfiType::Int8 => {
                    let idx = int8_idx;
                    int8_idx += 1;
                    Arg::new(&int8_args[idx])
                }
                FfiType::Int16 => {
                    let idx = int16_idx;
                    int16_idx += 1;
                    Arg::new(&int16_args[idx])
                }
                FfiType::Int32 => {
                    let idx = int32_idx;
                    int32_idx += 1;
                    Arg::new(&int32_args[idx])
                }
                FfiType::Int64 => {
                    let idx = int64_idx;
                    int64_idx += 1;
                    Arg::new(&int64_args[idx])
                }
                FfiType::UInt8 => {
                    let idx = uint8_idx;
                    uint8_idx += 1;
                    Arg::new(&uint8_args[idx])
                }
                FfiType::UInt16 => {
                    let idx = uint16_idx;
                    uint16_idx += 1;
                    Arg::new(&uint16_args[idx])
                }
                FfiType::UInt32 => {
                    let idx = uint32_idx;
                    uint32_idx += 1;
                    Arg::new(&uint32_args[idx])
                }
                FfiType::UInt64 => {
                    let idx = uint64_idx;
                    uint64_idx += 1;
                    Arg::new(&uint64_args[idx])
                }
                FfiType::Float => {
                    let idx = float_idx;
                    float_idx += 1;
                    Arg::new(&float_args[idx])
                }
                FfiType::Double => {
                    let idx = double_idx;
                    double_idx += 1;
                    Arg::new(&double_args[idx])
                }
                FfiType::Pointer => {
                    let idx = ptr_idx;
                    ptr_idx += 1;
                    Arg::new(&ptr_args[idx])
                }
                FfiType::String => {
                    let idx = cstring_idx;
                    cstring_idx += 1;
                    // Pass the pointer to the CString's data
                    let ptr = cstring_args[idx].as_ptr();
                    // We need to store this pointer somewhere
                    ptr_args.push(ptr as *const ());
                    Arg::new(&ptr_args[ptr_args.len() - 1])
                }
                FfiType::Void => unreachable!(),
            }
        }).collect();

        // Call the function
        let code_ptr = CodePtr::from_ptr(func_ptr as *const c_void);

        // Execute the call based on return type
        let result = unsafe {
            match return_type {
                FfiType::Void => {
                    cif.call::<()>(code_ptr, &ffi_args);
                    Value::List(super::value::List::Nil) // Return nil for void
                }
                FfiType::Int8 => {
                    let r: i8 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::Int16 => {
                    let r: i16 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::Int32 => {
                    let r: i32 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::Int64 => {
                    let r: i64 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r)
                }
                FfiType::UInt8 => {
                    let r: u8 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::UInt16 => {
                    let r: u16 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::UInt32 => {
                    let r: u32 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::UInt64 => {
                    let r: u64 = cif.call(code_ptr, &ffi_args);
                    Value::Integer(r as i64)
                }
                FfiType::Float => {
                    let r: f32 = cif.call(code_ptr, &ffi_args);
                    Value::Float(r as f64)
                }
                FfiType::Double => {
                    let r: f64 = cif.call(code_ptr, &ffi_args);
                    Value::Float(r)
                }
                FfiType::Pointer | FfiType::String => {
                    let r: *const () = cif.call(code_ptr, &ffi_args);
                    Value::Pointer(r as i64)
                }
            }
        };

        Ok(result)
    }

    // Helper: convert Value to i64
    fn value_to_int(&self, value: &Value) -> Result<i64, RuntimeError> {
        match value {
            Value::Integer(n) => Ok(*n),
            Value::Float(f) => Ok(*f as i64),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            Value::Pointer(p) => Ok(*p),
            _ => Err(RuntimeError::new(format!(
                "FFI: Cannot convert {} to integer",
                value_type_name(value)
            ))),
        }
    }

    // Helper: convert Value to f64
    fn value_to_float(&self, value: &Value) -> Result<f64, RuntimeError> {
        match value {
            Value::Float(f) => Ok(*f),
            Value::Integer(n) => Ok(*n as f64),
            _ => Err(RuntimeError::new(format!(
                "FFI: Cannot convert {} to float",
                value_type_name(value)
            ))),
        }
    }

    // Helper: convert Value to pointer
    fn value_to_pointer(&self, value: &Value) -> Result<i64, RuntimeError> {
        match value {
            Value::Pointer(p) => Ok(*p),
            Value::Integer(n) => Ok(*n), // Allow integers as pointers
            _ => Err(RuntimeError::new(format!(
                "FFI: Cannot convert {} to pointer",
                value_type_name(value)
            ))),
        }
    }

    // Helper: convert Value to string
    fn value_to_string(&self, value: &Value) -> Result<String, RuntimeError> {
        match value {
            Value::String(s) => Ok((**s).clone()),
            _ => Err(RuntimeError::new(format!(
                "FFI: Cannot convert {} to string",
                value_type_name(value)
            ))),
        }
    }
}

impl Default for FfiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert FfiType to libffi's native Type
fn ffi_type_to_native(ffi_type: &FfiType) -> FfiTypeNative {
    match ffi_type {
        FfiType::Void => FfiTypeNative::void(),
        FfiType::Int8 => FfiTypeNative::i8(),
        FfiType::Int16 => FfiTypeNative::i16(),
        FfiType::Int32 => FfiTypeNative::i32(),
        FfiType::Int64 => FfiTypeNative::i64(),
        FfiType::UInt8 => FfiTypeNative::u8(),
        FfiType::UInt16 => FfiTypeNative::u16(),
        FfiType::UInt32 => FfiTypeNative::u32(),
        FfiType::UInt64 => FfiTypeNative::u64(),
        FfiType::Float => FfiTypeNative::f32(),
        FfiType::Double => FfiTypeNative::f64(),
        FfiType::Pointer | FfiType::String => FfiTypeNative::pointer(),
    }
}

/// Get size of an FFI type in bytes
pub fn ffi_type_size(ffi_type: &FfiType) -> usize {
    match ffi_type {
        FfiType::Void => 0,
        FfiType::Int8 | FfiType::UInt8 => 1,
        FfiType::Int16 | FfiType::UInt16 => 2,
        FfiType::Int32 | FfiType::UInt32 | FfiType::Float => 4,
        FfiType::Int64 | FfiType::UInt64 | FfiType::Double | FfiType::Pointer | FfiType::String => 8,
    }
}

/// Helper to get type name for error messages
fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Integer(_) => "integer",
        Value::Float(_) => "float",
        Value::Boolean(_) => "boolean",
        Value::List(_) => "list",
        Value::Symbol(_) => "symbol",
        Value::String(_) => "string",
        Value::Function(_) => "function",
        Value::Closure(_) => "closure",
        Value::HashMap(_) => "hashmap",
        Value::Vector(_) => "vector",
        Value::TcpListener(_) => "tcp-listener",
        Value::TcpStream(_) => "tcp-stream",
        Value::SharedTcpListener(_) => "shared-tcp-listener",
        Value::Pointer(_) => "pointer",
    }
}

/// Parse an FFI type from a symbol string
pub fn parse_ffi_type(s: &str) -> Result<FfiType, String> {
    match s {
        ":void" | "void" => Ok(FfiType::Void),
        ":int8" | "int8" | ":char" | "char" => Ok(FfiType::Int8),
        ":int16" | "int16" | ":short" | "short" => Ok(FfiType::Int16),
        ":int32" | "int32" | ":int" | "int" => Ok(FfiType::Int32),
        ":int64" | "int64" | ":long" | "long" => Ok(FfiType::Int64),
        ":uint8" | "uint8" | ":uchar" | "uchar" => Ok(FfiType::UInt8),
        ":uint16" | "uint16" | ":ushort" | "ushort" => Ok(FfiType::UInt16),
        ":uint32" | "uint32" | ":uint" | "uint" => Ok(FfiType::UInt32),
        ":uint64" | "uint64" | ":ulong" | "ulong" => Ok(FfiType::UInt64),
        ":float" | "float" => Ok(FfiType::Float),
        ":double" | "double" => Ok(FfiType::Double),
        ":pointer" | "pointer" | ":ptr" | "ptr" | ":void*" | "void*" => Ok(FfiType::Pointer),
        ":string" | "string" | ":char*" | "char*" => Ok(FfiType::String),
        _ => Err(format!("Unknown FFI type: {}", s)),
    }
}
