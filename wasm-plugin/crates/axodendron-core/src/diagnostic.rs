use serde::{Deserialize, Serialize};

/// Validation behavior for real-world versus specification-conforming SWC data.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationProfile {
    /// Enforce the INCF SWC 1.0 ordering and single-root requirements.
    IncfStrict,
    /// Preserve out-of-order identifiers and rooted forests when structurally valid.
    #[default]
    Permissive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub node_id: Option<i64>,
}

impl Diagnostic {
    pub fn error(code: &str, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Error, message)
    }

    pub fn warning(code: &str, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Warning, message)
    }

    pub fn info(code: &str, message: impl Into<String>) -> Self {
        Self::new(code, Severity::Info, message)
    }

    fn new(code: &str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            line: None,
            column: None,
            node_id: None,
        }
    }

    pub fn at_line(mut self, line: u32) -> Self {
        self.line = Some(line);
        self
    }

    pub fn at_column(mut self, column: u32) -> Self {
        self.column = Some(column);
        self
    }

    pub fn for_node(mut self, id: i64) -> Self {
        self.node_id = Some(id);
        self
    }
}
