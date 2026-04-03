use crate::create_field::CreateField;

/// In-TUI form state for creating a new virtual environment
pub struct CreateDialog {
    /// Currently focused input field
    pub field: CreateField,
    pub name: String,
    pub version: String,
    /// Raw comma-separated packages string as the user types it
    pub packages: String,
    /// Optional path to a requirements.txt file to install on creation
    pub req_file: String,
    /// Cursor position (in characters) within `req_file`.
    pub req_file_cursor: usize,
    /// Directory completions (filtered by any prefix typed after the last `/`).
    pub completions: Vec<String>,
    /// Index of the currently highlighted completion entry.
    pub completion_selected: usize,
    /// First visible entry in the completion list (scroll offset).
    pub completion_scroll: usize,
    /// The directory portion of the path that was used to build `completions`
    /// (everything up to and including the last `/`).
    pub completions_dir: String,
    pub default_pkgs: bool,
}

impl CreateDialog {
    pub fn new(default_version: &str) -> Self {
        CreateDialog {
            field: CreateField::Name,
            name: String::new(),
            version: default_version.to_string(),
            packages: String::new(),
            req_file: String::new(),
            req_file_cursor: 0,
            completions: Vec::new(),
            completion_selected: 0,
            completion_scroll: 0,
            completions_dir: String::new(),
            default_pkgs: false,
        }
    }

    /// Push a character into the currently focused text field (no-op for bool field).
    /// For `ReqFile`, backslashes are normalized to forward slashes and the character
    /// is inserted at the current cursor position.
    pub fn push_char(&mut self, c: char) {
        match self.field {
            CreateField::Name => self.name.push(c),
            CreateField::Version => self.version.push(c),
            CreateField::Packages => self.packages.push(c),
            CreateField::ReqFile => {
                // Normalize Windows-style backslashes to forward slashes.
                let c = if c == '\\' { '/' } else { c };
                let byte_idx = self
                    .req_file
                    .char_indices()
                    .nth(self.req_file_cursor)
                    .map_or(self.req_file.len(), |(i, _)| i);
                self.req_file.insert(byte_idx, c);
                self.req_file_cursor += 1;
            }
            CreateField::DefaultPkgs => {}
        }
    }

    /// Delete the last character of the currently focused text field.
    /// For `ReqFile`, deletes the character immediately before the cursor (Backspace).
    pub fn pop_char(&mut self) {
        match self.field {
            CreateField::Name => {
                self.name.pop();
            }
            CreateField::Version => {
                self.version.pop();
            }
            CreateField::Packages => {
                self.packages.pop();
            }
            CreateField::ReqFile => {
                if self.req_file_cursor > 0 {
                    let byte_idx = self
                        .req_file
                        .char_indices()
                        .nth(self.req_file_cursor - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.req_file.remove(byte_idx);
                    self.req_file_cursor -= 1;
                }
            }
            CreateField::DefaultPkgs => {}
        }
    }

    /// Toggle the boolean default-packages field.
    pub fn toggle_default(&mut self) {
        self.default_pkgs = !self.default_pkgs;
    }

    /// Move the req_file cursor one character to the left.
    pub fn req_file_cursor_left(&mut self) {
        self.req_file_cursor = self.req_file_cursor.saturating_sub(1);
    }

    /// Move the req_file cursor one character to the right.
    pub fn req_file_cursor_right(&mut self) {
        let max = self.req_file.chars().count();
        if self.req_file_cursor < max {
            self.req_file_cursor += 1;
        }
    }

    /// Move the req_file cursor to the start of the input.
    pub fn req_file_cursor_home(&mut self) {
        self.req_file_cursor = 0;
    }

    /// Move the req_file cursor to the end of the input.
    pub fn req_file_cursor_end(&mut self) {
        self.req_file_cursor = self.req_file.chars().count();
    }

    /// Accept a completion entry: replaces everything after the last `/` in `req_file`
    /// with `entry`, using the stored `completions_dir` as the directory base.
    /// Updates the cursor to the new end of the string.
    pub fn req_file_accept_completion(&mut self, entry: &str) {
        self.req_file = format!("{}{}", self.completions_dir, entry);
        self.req_file_cursor = self.req_file.chars().count();
    }

    /// Collect packages from the raw comma-separated string.
    pub fn parsed_packages(&self) -> Vec<String> {
        self.packages
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Return the effective Python version: the user's input, or `DEFAULT_PYTHON_VERSION`
    /// if the version field was left blank.
    pub fn effective_version(&self) -> String {
        let v = self.version.trim();
        if v.is_empty() {
            pylot_shared::constants::DEFAULT_PYTHON_VERSION.to_string()
        } else {
            v.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dialog_new() {
        let d = CreateDialog::new("3.12");
        assert_eq!(d.field, CreateField::Name);
        assert_eq!(d.version, "3.12");
        assert!(d.name.is_empty());
        assert!(d.req_file.is_empty());
        assert_eq!(d.req_file_cursor, 0);
        assert!(d.completions.is_empty());
        assert_eq!(d.completion_selected, 0);
        assert!(!d.default_pkgs);
    }

    #[test]
    fn test_create_dialog_push_pop() {
        let mut d = CreateDialog::new("3.12");
        d.push_char('m');
        d.push_char('y');
        assert_eq!(d.name, "my");
        d.pop_char();
        assert_eq!(d.name, "m");
    }

    #[test]
    fn test_create_dialog_field_cycling() {
        assert_eq!(CreateField::Name.next(), CreateField::Version);
        assert_eq!(CreateField::Version.next(), CreateField::Packages);
        assert_eq!(CreateField::Packages.next(), CreateField::ReqFile);
        assert_eq!(CreateField::ReqFile.next(), CreateField::DefaultPkgs);
        assert_eq!(CreateField::DefaultPkgs.next(), CreateField::Name);

        assert_eq!(CreateField::Name.prev(), CreateField::DefaultPkgs);
        assert_eq!(CreateField::DefaultPkgs.prev(), CreateField::ReqFile);
        assert_eq!(CreateField::ReqFile.prev(), CreateField::Packages);
    }

    #[test]
    fn test_create_dialog_toggle_default() {
        let mut d = CreateDialog::new("3.12");
        assert!(!d.default_pkgs);
        d.toggle_default();
        assert!(d.default_pkgs);
        d.toggle_default();
        assert!(!d.default_pkgs);
    }

    #[test]
    fn test_create_dialog_parsed_packages() {
        let mut d = CreateDialog::new("3.12");
        d.packages = "requests, flask , ".to_string();
        assert_eq!(d.parsed_packages(), vec!["requests", "flask"]);
    }

    #[test]
    fn test_create_dialog_effective_version() {
        let d = CreateDialog::new("3.12");
        assert_eq!(d.effective_version(), "3.12");

        let mut d2 = CreateDialog::new("3.12");
        d2.version = "  ".to_string();
        // Blank version falls back to DEFAULT_PYTHON_VERSION.
        assert_eq!(
            d2.effective_version(),
            pylot_shared::constants::DEFAULT_PYTHON_VERSION
        );
    }

    #[test]
    fn test_create_dialog_pop_char_empty_does_not_panic() {
        let mut d = CreateDialog::new("3.12");
        // pop_char on an empty name field must not panic.
        d.pop_char();
        assert!(d.name.is_empty());
    }

    #[test]
    fn test_create_dialog_push_pop_version_field() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::Version;
        d.version.clear();
        d.push_char('3');
        d.push_char('.');
        d.push_char('9');
        assert_eq!(d.version, "3.9");
        d.pop_char();
        assert_eq!(d.version, "3.");
    }

    #[test]
    fn test_create_dialog_push_pop_packages_field() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::Packages;
        d.push_char('n');
        d.push_char('u');
        assert_eq!(d.packages, "nu");
        d.pop_char();
        assert_eq!(d.packages, "n");
    }

    #[test]
    fn test_create_dialog_push_pop_req_file_field() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.push_char('/');
        d.push_char('t');
        d.push_char('m');
        d.push_char('p');
        assert_eq!(d.req_file, "/tmp");
        assert_eq!(d.req_file_cursor, 4);
        d.pop_char();
        assert_eq!(d.req_file, "/tm");
        assert_eq!(d.req_file_cursor, 3);
    }

    #[test]
    fn test_create_dialog_req_file_backslash_normalized() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.push_char('C');
        d.push_char(':');
        d.push_char('\\');
        d.push_char('f');
        // Backslash should be stored as forward slash.
        assert_eq!(d.req_file, "C:/f");
    }

    #[test]
    fn test_create_dialog_req_file_cursor_left_right() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.push_char('a');
        d.push_char('b');
        d.push_char('c');
        assert_eq!(d.req_file_cursor, 3);
        d.req_file_cursor_left();
        assert_eq!(d.req_file_cursor, 2);
        d.req_file_cursor_right();
        assert_eq!(d.req_file_cursor, 3);
        // Cannot go past end.
        d.req_file_cursor_right();
        assert_eq!(d.req_file_cursor, 3);
    }

    #[test]
    fn test_create_dialog_req_file_cursor_left_clamp() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.req_file_cursor_left(); // must not underflow
        assert_eq!(d.req_file_cursor, 0);
    }

    #[test]
    fn test_create_dialog_req_file_cursor_home_end() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.push_char('/');
        d.push_char('t');
        d.push_char('m');
        d.push_char('p');
        assert_eq!(d.req_file_cursor, 4);
        d.req_file_cursor_home();
        assert_eq!(d.req_file_cursor, 0);
        d.req_file_cursor_end();
        assert_eq!(d.req_file_cursor, 4);
    }

    #[test]
    fn test_create_dialog_req_file_insert_mid() {
        // Inserting a character mid-string when cursor is not at the end.
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.push_char('a');
        d.push_char('c'); // "ac", cursor = 2
        d.req_file_cursor_left(); // cursor = 1
        d.push_char('b'); // insert 'b' at pos 1 → "abc", cursor = 2
        assert_eq!(d.req_file, "abc");
        assert_eq!(d.req_file_cursor, 2);
    }

    #[test]
    fn test_create_dialog_req_file_accept_completion_appends_to_dir() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        // Simulate the state after update_completions set completions_dir.
        d.req_file = "/tmp/req".to_string();
        d.req_file_cursor = 8;
        d.completions_dir = "/tmp/".to_string();
        d.req_file_accept_completion("requirements.txt");
        assert_eq!(d.req_file, "/tmp/requirements.txt");
        assert_eq!(d.req_file_cursor, d.req_file.chars().count());
    }

    #[test]
    fn test_create_dialog_req_file_accept_completion_dir_then_re_enter() {
        // Accepting a directory entry should leave a trailing '/' so completions
        // can be re-triggered for the next level.
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::ReqFile;
        d.req_file = "/tmp/".to_string();
        d.req_file_cursor = 5;
        d.completions_dir = "/tmp/".to_string();
        d.req_file_accept_completion("subdir/");
        assert_eq!(d.req_file, "/tmp/subdir/");
        assert_eq!(d.req_file_cursor, d.req_file.chars().count());
    }

    #[test]
    fn test_create_dialog_push_char_default_pkgs_noop() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::DefaultPkgs;
        // push_char should be a no-op on the bool field.
        d.push_char('x');
        d.push_char('y');
        assert!(d.name.is_empty());
        assert!(d.packages.is_empty());
    }

    #[test]
    fn test_create_dialog_pop_char_default_pkgs_noop() {
        let mut d = CreateDialog::new("3.12");
        d.field = CreateField::DefaultPkgs;
        // pop_char should be a no-op on the bool field.
        d.pop_char();
        assert!(!d.default_pkgs); // unchanged
    }
}
