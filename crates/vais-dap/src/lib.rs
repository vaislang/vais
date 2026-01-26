//! Vais Debug Adapter Protocol (DAP) Server
//!
//! This crate implements the Debug Adapter Protocol for Vais programs,
//! enabling IDE-level debugging support in editors like VSCode, Neovim, and Emacs.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │       DAP Server (this crate)           │
//! ├─────────────────────────────────────────┤
//! │ • Protocol handling (JSON-RPC)          │
//! │ • Session management                    │
//! │ • Breakpoint management                 │
//! │ • Stack trace / Variables               │
//! ├─────────────────────────────────────────┤
//! │   Debugger Backend (LLDB)               │
//! ├─────────────────────────────────────────┤
//! │ • Process control                       │
//! │ • Event handling                        │
//! │ • Memory inspection                     │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Start the DAP server (stdio mode)
//! vais-dap
//!
//! # Start with TCP socket
//! vais-dap --port 4711
//! ```

pub mod protocol;
pub mod server;
pub mod session;
pub mod debugger;
pub mod breakpoint;
pub mod stack;
pub mod variables;
pub mod source_map;
pub mod error;

pub use error::{DapError, DapResult};
pub use server::DapServer;
pub use session::DebugSession;
pub use protocol::types;
