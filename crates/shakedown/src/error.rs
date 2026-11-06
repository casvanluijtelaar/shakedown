//! An error the command line reports: a message that says what went wrong and what to do next
//! (`NFR-US-01`), and the exit code of `contract/cli.md`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliError {
    pub code: u8,
    pub message: String,
    /// The error code of the MCP contract (`contract/mcp-tools.md`), such as `unknown-machine`,
    /// for an error the AI agent can fix; `None` takes the code from the exit code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl CliError {
    /// Exit code 3: the request is wrong.
    pub fn wrong(message: impl Into<String>) -> Self {
        CliError {
            code: 3,
            message: message.into(),
            tag: None,
        }
    }

    /// Exit code 4: the request is right but cannot be done now.
    pub fn not_now(message: impl Into<String>) -> Self {
        CliError {
            code: 4,
            message: message.into(),
            tag: None,
        }
    }

    /// Exit code 5: the framework itself failed.
    pub fn framework(message: impl Into<String>) -> Self {
        CliError {
            code: 5,
            message: message.into(),
            tag: None,
        }
    }

    /// Gives the error the MCP contract's code.
    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// The MCP contract's code: the tag, or for an untagged error what its exit code means;
    /// `None` for a failure of the framework, which the agent cannot fix.
    pub fn mcp_code(&self) -> Option<&str> {
        match (&self.tag, self.code) {
            (Some(t), _) => Some(t),
            (None, 3) => Some("bad-arguments"),
            (None, 4) => Some("session-busy"),
            _ => None,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}
