use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub message: String,
    pub span: Span,
}

impl SyntaxError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self { message: message.into(), span }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error at {}-{}: {}", self.span.start, self.span.end, self.message)
    }
}

impl std::error::Error for SyntaxError {}

pub fn format_error(source: &str, err: &SyntaxError) -> String {
    let mut line_num = 1;
    let mut line_start = 0;
    for (i, ch) in source.char_indices() {
        if i >= err.span.start { break; }
        if ch == '\n' { line_num += 1; line_start = i + 1; }
    }
    let line_end = source[line_start..].find('\n').map(|i| line_start + i).unwrap_or(source.len());
    let line_text = &source[line_start..line_end];
    let col = err.span.start.saturating_sub(line_start);
    let underline_len = (err.span.end - err.span.start).max(1).min(line_text.len().saturating_sub(col));
    format!(
        "error[line {}]: {}\n  |\n{} | {}\n  | {}{}\n",
        line_num, err.message, line_num, line_text,
        " ".repeat(col), "^".repeat(underline_len),
    )
}
