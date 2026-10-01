//! Sedimentary: a virtual pet rock that is deeply unimpressed with its owner.
//!
//! Module map:
//! - [`rock`]: the pet itself. Stats, actions, drift, offline catch-up. No text, no I/O.
//! - [`achievements`]: hidden discoveries and their descriptions.
//! - [`lines`] + [`responses`]: everything the rock says, and how lines are chosen.
//! - [`journal`]: the field notes of recent interactions.
//! - [`save`]: JSON persistence with corruption recovery.
//! - [`app`]: screens, input handling, timing. Glues the pieces together.
//! - [`art`] + [`ui`]: the text-art rock and the Ratatui interface.

pub mod achievements;
pub mod app;
pub mod art;
pub mod clock;
pub mod dice;
pub mod journal;
pub mod lines;
pub mod responses;
pub mod rock;
pub mod save;
pub mod ui;
