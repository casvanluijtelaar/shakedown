//! `shakedown_x64.dll`: the extension that connects its game process to the runner (`INT-AR-03`).
//!
//! It only moves bytes. Orders arrive from the runner on a thread of its own, are queued, and the
//! in-game agent is told through the extension callback; the agent fetches each order in parts and
//! sends messages back with `send`, which queues and returns at once. No call does more than that,
//! because the extension cannot be replaced while the game runs (`NFR-MT-01`).

mod bridge;

use std::sync::{Arc, OnceLock};

use arma_rs::{Context, Extension, arma};

pub use bridge::{Bridge, Env, Link};

static BRIDGE: OnceLock<Arc<Bridge>> = OnceLock::new();

#[arma]
fn init() -> Extension {
    Extension::build()
        .version(shakedown_protocol::VERSION.to_string())
        .command("fetch", fetch)
        .command("send", send)
        .command("status", status)
        .finish()
}

/// The bridge, connecting it on the first call with the callback of that call's context.
fn bridge(ctx: Context) -> &'static Arc<Bridge> {
    BRIDGE.get_or_init(|| {
        let bridge = Arc::new(Bridge::new());
        let env = Env::from_process();
        bridge.start(
            env,
            Box::new(move |id: &str| {
                let _ = ctx.callback_data("shakedown", "order", id.to_string());
            }),
        );
        bridge
    })
}

/// `"shakedown" callExtension ["fetch", [orderId, part]]`: one part of a queued order, prefixed
/// with `1` when it is the last part and `0` when more follow.
fn fetch(ctx: Context, id: String, part: u32) -> Result<String, String> {
    bridge(ctx).fetch(&id, part as usize)
}

/// `"shakedown" callExtension ["send", [json]]`: queues a message for the runner.
fn send(ctx: Context, json: String) -> &'static str {
    if bridge(ctx).send(json) { "ok" } else { "no-runner" }
}

/// `"shakedown" callExtension ["status", []]`: the connection, the session id and the machine.
fn status(ctx: Context) -> Vec<String> {
    let (link, session, machine) = bridge(ctx).status();
    vec![link.as_str().to_string(), session, machine]
}
