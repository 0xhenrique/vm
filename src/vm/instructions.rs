use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Push(Value),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Leq,
    Lt,
    Gt,
    Gte,
    Eq,
    Neq,
    JmpIfFalse(usize),
    Jmp(usize),
    Call(String, usize),
    TailCall(String, usize), // Tail call: reuse current frame instead of pushing new one
    Ret,
    LoadArg(usize),
    GetLocal(usize), // Load from value stack at position (from bottom)
    PopN(usize),     // Pop N values from the stack
    Slide(usize),    // Pop top value, pop N values, push top value back (cleanup let bindings)
    CheckArity(usize, usize), // Check if frame.locals.len() == expected_arity, jump to addr if not
    PackRestArgs(usize), // Collect args from index N onwards into a list, replace them with the list in frame.locals
    MakeClosure(Vec<String>, Vec<Instruction>, usize), // Create closure: (params, body, num_captured_vars)
    MakeVariadicClosure(Vec<String>, String, Vec<Instruction>, usize), // Variadic closure: (required_params, rest_param, body, num_captured)
    CallClosure(usize), // Call closure with N arguments (pops closure + args from stack)
    Apply,              // Apply function to list of arguments: pop list, pop function/closure, call with list elements as args
    LoadCaptured(usize), // Load captured variable at index from current closure's environment
    Print,
    Halt,
    // List operations
    Cons,    // Pop two values, push cons cell (list)
    Car,     // Pop list, push first element
    Cdr,     // Pop list, push rest of list
    IsList,  // Pop value, push boolean indicating if it's a list
    // Type predicates
    IsInteger,      // Pop value, push boolean indicating if it's an integer
    IsBoolean,      // Pop value, push boolean indicating if it's a boolean
    IsFunction,     // Pop value, push boolean indicating if it's a function
    IsClosure,      // Pop value, push boolean indicating if it's a closure
    IsProcedure,    // Pop value, push boolean indicating if it's a function or closure
    // String/Symbol operations
    IsString,       // Pop value, push boolean indicating if it's a string
    IsSymbol,       // Pop value, push boolean indicating if it's a symbol
    SymbolToString, // Pop symbol, push string
    StringToSymbol, // Pop string, push symbol
    StringLength,   // Pop string, push integer length
    Substring,      // Pop string, start, end; push substring
    StringAppend,   // Pop two strings, push concatenation
    StringToList,   // Pop string, push list of single-char strings
    ListToString,   // Pop list of strings/chars, push concatenated string
    CharCode,       // Pop single-char string, push ASCII code as integer
    // List manipulation
    Append,         // Pop two lists, push their concatenation (second appended to first)
    MakeList(usize), // Pop N values from stack and create a list from them (in order)
    ListRef,        // Pop list and index, push element at that index (0-based)
    ListLength,     // Pop list, push its length as integer
    // Number operations
    NumberToString, // Pop integer, push string representation
    StringToNumber, // Pop string, push integer (or error if not a valid number)
    // File I/O operations
    ReadFile,       // Pop string path, push file contents as string (or error)
    WriteFile,      // Pop string path, string content; push boolean success
    FileExists,     // Pop string path, push boolean indicating if file exists
    WriteBinaryFile, // Pop string path, list of integers (bytes); write binary file
    LoadFile,       // Pop string path, load and execute Lisp file in current environment
    RequireFile,    // Pop string path, load and execute Lisp file only if not already loaded
    // Global variables
    LoadGlobal(String),  // Push value of global variable onto stack
    StoreGlobal(String), // Pop value from stack and store in global variable
    // Command-line arguments
    GetArgs,             // Push command-line arguments as a list of strings
    // HashMap operations
    MakeHashMap(usize),  // Pop N key-value pairs from stack (key1, val1, key2, val2, ...) and create a hashmap
    HashMapGet,          // Pop hashmap and key, push value (or error if not found)
    HashMapSet,          // Pop hashmap, key, value; push new hashmap with key-value set
    HashMapKeys,         // Pop hashmap, push list of keys
    HashMapValues,       // Pop hashmap, push list of values
    HashMapContainsKey,  // Pop hashmap and key, push boolean
    IsHashMap,           // Pop value, push boolean indicating if it's a hashmap
    // Vector operations
    MakeVector(usize),   // Pop N values from stack and create a vector from them (in order)
    VectorGet,           // Pop vector and index, push element at that index (0-based)
    VectorSet,           // Pop vector, index, value; push new vector with element at index set
    VectorPush,          // Pop vector and value, push new vector with value appended
    VectorPop,           // Pop vector, push vector without last element and the last element
    VectorLength,        // Pop vector, push its length as integer
    IsVector,            // Pop value, push boolean indicating if it's a vector
    // Type conversions
    ListToVector,        // Pop list, push vector with same elements
    VectorToList,        // Pop vector, push list with same elements
}
