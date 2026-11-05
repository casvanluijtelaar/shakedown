//! Messages between the Shakedown runner, its clients and its extension, and the types of the
//! result file (`spec/contract/result-file.md`).

pub mod result;
pub mod time;

use serde::{Deserialize, Serialize};

/// The framework version, shared by the binary and the extension (one build, one version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The environment variables the runner gives every game process (`SEC-02`).
pub const ENV_RUNNER: &str = "SHAKEDOWN_RUNNER";
pub const ENV_SESSION: &str = "SHAKEDOWN_SESSION";
pub const ENV_MACHINE: &str = "SHAKEDOWN_MACHINE";
pub const ENV_TOKEN: &str = "SHAKEDOWN_TOKEN";

/// The largest part of an order the extension returns from one `fetch` (`INT-AR-03`).
pub const PART_SIZE: usize = 10_000;

/// The first message on a connection to the runner (`contract/cli.md`, "The runner's file and connection").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "hello", rename_all = "lowercase")]
pub enum Hello {
    Client {
        version: String,
        token: String,
    },
    Extension {
        session: String,
        machine: String,
        token: String,
    },
    Runner {
        version: String,
    },
}

/// Splits `text` into parts of at most `max` bytes, each ending on a character boundary.
/// An empty text is one empty part, so a fetch always has a last part.
pub fn split_parts(text: &str, max: usize) -> Vec<&str> {
    assert!(max >= 4, "a part must hold any one UTF-8 character");
    let mut parts = Vec::new();
    let mut rest = text;
    while rest.len() > max {
        let mut cut = max;
        while !rest.is_char_boundary(cut) {
            cut -= 1;
        }
        let (head, tail) = rest.split_at(cut);
        parts.push(head);
        rest = tail;
    }
    parts.push(rest);
    parts
}

/// Reads the id of an order line without parsing the whole order.
pub fn order_id(line: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct Id {
        id: String,
    }
    serde_json::from_str::<Id>(line).ok().map(|o| o.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_ar_03_parts_are_at_most_the_part_size_and_rejoin() {
        let text = "é".repeat(15_001);
        let parts = split_parts(&text, PART_SIZE);
        assert!(parts.iter().all(|p| p.len() <= PART_SIZE));
        assert_eq!(parts.concat(), text);
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn int_ar_03_empty_text_is_one_last_part() {
        assert_eq!(split_parts("", PART_SIZE), vec![""]);
    }

    #[test]
    fn hello_has_the_contract_shape() {
        let hello = Hello::Extension {
            session: "s-20261105-120000".into(),
            machine: "p1".into(),
            token: "skd_s_00".into(),
        };
        let json = serde_json::to_value(&hello).unwrap();
        assert_eq!(json["hello"], "extension");
        assert_eq!(json["machine"], "p1");
        let back: Hello = serde_json::from_value(json).unwrap();
        assert_eq!(back, hello);
    }

    #[test]
    fn order_id_reads_the_id() {
        assert_eq!(order_id(r#"{"id":"o7","kind":"exec"}"#).as_deref(), Some("o7"));
        assert_eq!(order_id("not json"), None);
    }
}
