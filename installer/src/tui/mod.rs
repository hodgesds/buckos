//! Text User Interface (TUI) for BuckOS installer
//!
//! This module provides a terminal-based user interface for installing BuckOS
//! using ratatui and crossterm.

mod app;
mod widgets;

pub use app::run_tui_installer;
