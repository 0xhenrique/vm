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

    pub fn format(&self) -> String {
        if self.file == "<unknown>" {
            String::from("<unknown>")
        } else {
            format!("{}:{}:{}", self.file, self.line, self.column)
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

    /// Format the error with optional source code context
    /// If source is provided, it should be the full source file content
    pub fn format(&self, source: Option<&str>) -> String {
        let mut output = String::new();

        // Header with location
        output.push_str(&format!("\n╭─ Compile Error ─────────────────────────────\n"));
        output.push_str(&format!("│ {}\n", self.location.format()));
        output.push_str(&format!("├─────────────────────────────────────────────\n"));
        output.push_str(&format!("│ {}\n", self.message));

        // Show source context if available
        if let Some(src) = source {
            let lines: Vec<&str> = src.lines().collect();
            if self.location.line > 0 && self.location.line <= lines.len() {
                output.push_str(&format!("├─────────────────────────────────────────────\n"));

                // Show line before (if exists)
                if self.location.line > 1 {
                    output.push_str(&format!(
                        "│ {:4} │ {}\n",
                        self.location.line - 1,
                        lines[self.location.line - 2]
                    ));
                }

                // Show error line with pointer
                let error_line = lines[self.location.line - 1];
                output.push_str(&format!(
                    "│ {:4} │ {}\n",
                    self.location.line,
                    error_line
                ));

                // Add pointer to error location
                let pointer_padding = self.location.column.saturating_sub(1);
                output.push_str(&format!(
                    "│      │ {}^\n",
                    " ".repeat(pointer_padding)
                ));

                // Show line after (if exists)
                if self.location.line < lines.len() {
                    output.push_str(&format!(
                        "│ {:4} │ {}\n",
                        self.location.line + 1,
                        lines[self.location.line]
                    ));
                }
            }
        }

        output.push_str(&format!("╰─────────────────────────────────────────────\n"));
        output
    }

    /// Simple format without source context (for backward compatibility)
    pub fn format_simple(&self) -> String {
        format!(
            "Compile error at {}: {}",
            self.location.format(),
            self.message
        )
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub call_stack: Vec<String>,
    pub location: Option<Location>,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        RuntimeError {
            message,
            call_stack: Vec::new(),
            location: None,
        }
    }

    pub fn with_stack(message: String, call_stack: Vec<String>) -> Self {
        RuntimeError {
            message,
            call_stack,
            location: None,
        }
    }

    pub fn with_location(message: String, location: Location) -> Self {
        RuntimeError {
            message,
            call_stack: Vec::new(),
            location: Some(location),
        }
    }

    pub fn with_stack_and_location(
        message: String,
        call_stack: Vec<String>,
        location: Option<Location>,
    ) -> Self {
        RuntimeError {
            message,
            call_stack,
            location,
        }
    }

    pub fn format(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("\n╭─ Runtime Error ─────────────────────────────\n");

        // Show location if available
        if let Some(loc) = &self.location {
            output.push_str(&format!("│ {}\n", loc.format()));
            output.push_str("├─────────────────────────────────────────────\n");
        }

        // Error message
        output.push_str(&format!("│ {}\n", self.message));

        // Call stack
        if !self.call_stack.is_empty() {
            output.push_str("├─ Call Stack ────────────────────────────────\n");
            for (i, frame) in self.call_stack.iter().rev().enumerate() {
                output.push_str(&format!("│ #{}: {}\n", i, frame));
            }
        }

        output.push_str("╰─────────────────────────────────────────────\n");
        output
    }

    pub fn format_simple(&self) -> String {
        let loc_str = self
            .location
            .as_ref()
            .map(|l| format!(" at {}", l.format()))
            .unwrap_or_default();
        format!("Runtime error{}: {}", loc_str, self.message)
    }
}
