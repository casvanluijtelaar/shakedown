//! An error the command line reports: a message that says what went wrong and what to do next
//! (`NFR-US-01`), and the exit code of `contract/cli.md`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliError {
    pub code: u8,
    pub message: String,
}

impl CliError {
    /// Exit code 3: the request is wrong.
    pub fn wrong(message: impl Into<String>) -> Self {
        CliError {
            code: 3,
            message: message.into(),
        }
    }

    /// Exit code 4: the request is right but cannot be done now.
    pub fn not_now(message: impl Into<String>) -> Self {
        CliError {
            code: 4,
            message: message.into(),
        }
    }

    /// Exit code 5: the framework itself failed.
    pub fn framework(message: impl Into<String>) -> Self {
        CliError {
            code: 5,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}
