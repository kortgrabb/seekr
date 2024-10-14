
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub file: String,                 // Name of the file containing the line
    pub line_number: usize,           // Line number in the file
    pub line_content: String,         // Content of the line
    pub matches: Vec<(usize, usize)>, // (start, end) positions of matches           // Indicates if this is an inverted match
}

impl SearchMatch {
    // Updated constructor to include `is_inverted`
    pub fn new(
        file: &str,
        line_number: usize,
        line_content: String,
        matches: Vec<(usize, usize)>,
    ) -> Self {
        Self {
            file: file.to_owned(),
            line_number,
            line_content,
            matches,
        }
    }
}
