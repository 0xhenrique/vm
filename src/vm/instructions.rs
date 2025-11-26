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
    MakeClosure(Vec<String>, Vec<Instruction>, usize), // Create closure: (params, body, num_captured_vars)
    CallClosure(usize), // Call closure with N arguments (pops closure + args from stack)
    LoadCaptured(usize), // Load captured variable at index from current closure's environment
    Print,
    Halt,
    // List operations
    Cons,    // Pop two values, push cons cell (list)
    Car,     // Pop list, push first element
    Cdr,     // Pop list, push rest of list
    IsList,  // Pop value, push boolean indicating if it's a list
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
    // File I/O operations
    ReadFile,       // Pop string path, push file contents as string (or error)
    WriteFile,      // Pop string path, string content; push boolean success
    FileExists,     // Pop string path, push boolean indicating if file exists
    WriteBinaryFile, // Pop string path, list of integers (bytes); write binary file
    // Global variables
    LoadGlobal(String),  // Push value of global variable onto stack
    StoreGlobal(String), // Pop value from stack and store in global variable
    // Command-line arguments
    GetArgs,             // Push command-line arguments as a list of strings
}
