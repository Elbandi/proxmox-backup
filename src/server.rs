//! Proxmox Server/Service framework
//!
//! This code provides basic primitives to build our REST API
//! services. We want async IO, so this is built on top of
//! tokio/hyper.

mod environment;
pub use environment::*;

mod state;
pub use state::*;

mod command_socket;
pub use command_socket::*;

mod worker_task;
pub use worker_task::*;

pub mod formatter;

#[macro_use]
pub mod rest;

