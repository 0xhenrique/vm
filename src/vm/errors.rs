#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl Location {
    pub fn new(line: usize, column: usize, file: String) -> Self {
        Location { line, column, file }
    }

    pub fn unknown() -> Self {
        Location {
            line: 0,
            column: 0,
            file: String::from("<unknown>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub location: Location,
}

impl CompileError {
    pub fn new(message: String, location: Location) -> Self {
        CompileError { message, location }
    }

    pub fn format(&self, source_line: Option<&str>) -> String {
        let mut output = format!(
            "Compile error at {}:{}:{}\n  {}\n",
            self.location.file, self.location.line, self.location.column, self.message
        );

        if let Some(line) = source_line {
            output.push_str(&format!("\n  | {}\n", line));
            output.push_str(&format!("  | {}^\n", " ".repeat(self.location.column.saturating_sub(1))));
        }

        output
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub call_stack: Vec<String>,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        RuntimeError {
            message,
            call_stack: Vec::new(),
        }
    }

    pub fn with_stack(message: String, call_stack: Vec<String>) -> Self {
        RuntimeError {
            message,
            call_stack,
        }
    }

    pub fn format(&self) -> String {
        let mut output = format!("Runtime error: {}\n", self.message);

        if !self.call_stack.is_empty() {
            output.push_str("\nCall stack:\n");
            for (i, frame) in self.call_stack.iter().rev().enumerate() {
                output.push_str(&format!("  #{}: {}\n", i, frame));
            }
        }

        output
    }
}
