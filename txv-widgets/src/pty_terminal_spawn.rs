//! PtyTerminal constructors — spawn shell or command processes.

use std::env;
use std::path::Path;

use txv_core::prelude::*;
use txv_render::termbuf::TermBuf;

use crate::pty_session::PtySession;
use crate::pty_terminal::PtyTerminal;

impl PtyTerminal {
    /// Spawn the user's default shell.
    pub fn spawn_shell(cols: u16, rows: u16) -> std::io::Result<Self> {
        Self::spawn_shell_with_scrollback(cols, rows, 2000)
    }

    /// Spawn the user's default shell with a custom scrollback limit.
    pub fn spawn_shell_with_scrollback(cols: u16, rows: u16, scrollback_limit: usize) -> std::io::Result<Self> {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let cwd = env::current_dir().unwrap_or_else(|_| "/".into());
        let session = PtySession::spawn(&shell, &[], &cwd, cols, rows)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::with_scrollback(cols, rows, scrollback_limit),
            session: Some(session),
            base_title: "Shell".into(),
            title: "Shell".into(),
            osc_suffix: String::new(),
            prev_cols: cols,
            prev_rows: rows,
            exited: false,
            scroll_offset: 0,
            had_output: false,
        })
    }

    /// Spawn a specific command.
    pub fn spawn_command(cmd: &str, args: &[&str], cwd: &Path, cols: u16, rows: u16) -> std::io::Result<Self> {
        Self::spawn_command_with_scrollback(cmd, args, cwd, cols, rows, 2000)
    }

    /// Spawn a specific command with additional environment variables.
    pub fn spawn_command_with_env(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        cols: u16,
        rows: u16,
        envs: &[(&str, &str)],
    ) -> std::io::Result<Self> {
        let session = PtySession::spawn_with_env(cmd, args, cwd, cols, rows, envs)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::with_scrollback(cols, rows, 2000),
            session: Some(session),
            base_title: cmd.into(),
            title: cmd.into(),
            osc_suffix: String::new(),
            prev_cols: cols,
            prev_rows: rows,
            exited: false,
            scroll_offset: 0,
            had_output: false,
        })
    }

    /// Spawn a specific command with a custom scrollback limit.
    pub fn spawn_command_with_scrollback(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        cols: u16,
        rows: u16,
        scrollback_limit: usize,
    ) -> std::io::Result<Self> {
        let session = PtySession::spawn(cmd, args, cwd, cols, rows)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::with_scrollback(cols, rows, scrollback_limit),
            session: Some(session),
            base_title: cmd.into(),
            title: cmd.into(),
            osc_suffix: String::new(),
            prev_cols: cols,
            prev_rows: rows,
            exited: false,
            scroll_offset: 0,
            had_output: false,
        })
    }
}
