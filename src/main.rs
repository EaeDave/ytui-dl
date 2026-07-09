mod action;
mod app;
mod config;
mod downloader;
mod error;
mod event;
mod i18n;
mod models;
mod tui;
mod ui;
mod updater;

use std::io::{self, IsTerminal, Write};

use color_eyre::eyre::{bail, Result, WrapErr};
use tokio::sync::mpsc;

use crate::action::Action;
use crate::app::App;
use crate::config::Config;
use crate::event::EventHandler;
use crate::tui::Tui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_version() {
    println!("ytui-dl {VERSION}");
}

fn print_cli_help() {
    println!(
        "\
ytui-dl {VERSION} — YouTube TUI downloader

Usage:
  ytui-dl              Start the TUI
  ytui-dl --version    Print version
  ytui-dl --update     Download and install the latest GitHub release
  ytui-dl --update --force
                       Reinstall even if already on the latest version
  ytui-dl --uninstall  Remove the installed binary (keeps config/downloads)
  ytui-dl --help       Print this help

First-time install:
  Linux:   curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
  Windows: irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex
"
    );
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Always surface errors — some Windows hosts swallow panics poorly.
        eprintln!("ytui-dl error: {e:#}");
        let _ = io::stderr().flush();
        // Keep console visible briefly when double-started; no-op in normal terminals.
        #[cfg(windows)]
        {
            if std::env::var_os("YTUI_NO_PAUSE").is_none() && !io::stdin().is_terminal() {
                // non-interactive
            }
        }
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(arg) = args.first().map(|s| s.as_str()) {
        match arg {
            "-V" | "--version" | "version" => {
                print_version();
                return Ok(());
            }
            "-h" | "--help" | "help" => {
                print_cli_help();
                return Ok(());
            }
            "--update" | "update" => {
                let force = args.iter().any(|a| a == "--force" || a == "-f");
                updater::run_self_update(force)
                    .await
                    .wrap_err("update failed")?;
                return Ok(());
            }
            "--uninstall" | "uninstall" => {
                updater::run_uninstall().wrap_err("uninstall failed")?;
                return Ok(());
            }
            other => {
                bail!("unknown argument: {other}\ntry: ytui-dl --help");
            }
        }
    }

    // Interactive TUI requires a real terminal (not a pipe / background job).
    if !io::stdout().is_terminal() {
        bail!(
            "stdout is not an interactive terminal.\n\
             Open Windows Terminal (or conhost) and run: ytui-dl\n\
             Or check the binary with: ytui-dl --version"
        );
    }

    color_eyre::install().wrap_err("install error handler")?;

    let config = Config::load();
    let mut app = App::new(config);
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    app.set_action_tx(action_tx.clone());

    // Background: notify if a newer GitHub release exists (never blocks UI).
    updater::spawn_check(action_tx.clone());

    let mut tui = Tui::new().wrap_err("init TUI")?;
    tui.enter().wrap_err(
        "enter TUI mode — on Windows, prefer Windows Terminal over legacy console",
    )?;

    // Keep the event handler alive for the whole session (do not prefix with `_` alone in a way that drops).
    let _event_handler = EventHandler::new(action_tx.clone(), 4.0, 30.0);

    // Initial render
    tui.terminal()
        .draw(|frame| ui::draw(frame, &app))
        .wrap_err("initial draw")?;

    // Keep sender alive so the channel never closes while the UI runs.
    let _keep_tx = action_tx;

    loop {
        let Some(action) = action_rx.recv().await else {
            // Should not happen while _keep_tx lives; treat as fatal.
            bail!("event channel closed unexpectedly");
        };

        app.update(action).wrap_err("app update")?;

        if app.should_quit {
            break;
        }

        tui.terminal()
            .draw(|frame| ui::draw(frame, &app))
            .wrap_err("draw frame")?;
    }

    let should_restart = app.should_restart;
    let restart_path = app.restart_path.clone();
    tui.exit().wrap_err("leave TUI")?;

    if should_restart {
        if let Err(e) = updater::reexec_self(restart_path) {
            eprintln!("error: could not restart: {e:#}");
            eprintln!("launch manually: ytui-dl");
            std::process::exit(1);
        }
    }

    Ok(())
}
