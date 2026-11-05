//! Shakedown: test automation for Arma 3 mods. One binary holds the runner and the command line
//! (`spec/07-tech-stack.md`, "Shape").

pub mod checks;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod mission;
pub mod paths;
pub mod runner;
pub mod win;
pub mod wire;
