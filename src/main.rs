mod action;
mod app;
mod config;
mod diag;
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
    println!("ytd {VERSION}");
}

fn print_cli_help() {
    println!(
        "\
ytd {VERSION} — YouTube TUI downloader (formerly ytui-dl)

Usage:
  ytd              Start the TUI
  ytd --version    Print version
  ytd --doctor     Self-check (TTY, raw mode, PATH tools) + write last-run.log
  ytd --update     Download and install the latest GitHub release
  ytd --update --force
                   Reinstall even if already on the latest version
  ytd --uninstall  Remove the installed binary (keeps config/downloads)
  ytd --help       Print this help

  Alias: ytui-dl  (same binary; still installed for compatibility)

First-time install:
  Linux:   curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
  Windows: irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex

If the TUI opens and closes with no message, run:
  ytd --doctor
  type %LOCALAPPDATA%\\ytui-dl\\last-run.log     (Windows)
  cat ~/.local/share/ytui-dl/last-run.log       (Linux)
"
    );
}

#[tokio::main]
async fn main() {
    // Catch panics that would otherwise vanish on some Windows hosts.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        diag::breadcrumb(&format!("PANIC: {info}"));
        eprintln!("ytd panic: {info}");
        let _ = io::stderr().flush();
        default_hook(info);
    }));

    if let Err(e) = run().await {
        diag::breadcrumb(&format!("fatal error: {e:#}"));
        // Always surface errors — some Windows hosts swallow panics poorly.
        eprintln!("ytd error: {e:#}");
        eprintln!("(details: {})", diag::log_path().display());
        let _ = io::stderr().flush();
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    diag::begin_run(&args);

    if let Some(arg) = args.first().map(|s| s.as_str()) {
        match arg {
            "-V" | "--version" | "version" => {
                print_version();
                diag::breadcrumb("version ok");
                return Ok(());
            }
            "-h" | "--help" | "help" => {
                print_cli_help();
                return Ok(());
            }
            "--doctor" | "doctor" => {
                diag::run_doctor()?;
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
                bail!("unknown argument: {other}\ntry: ytd --help");
            }
        }
    }

    // Interactive TUI requires a real terminal (not a pipe / background job).
    let stdout_tty = io::stdout().is_terminal();
    let stdin_tty = io::stdin().is_terminal();
    diag::breadcrumb(&format!(
        "tty check stdin={stdin_tty} stdout={stdout_tty}"
    ));
    if !stdout_tty {
        bail!(
            "stdout is not an interactive terminal.\n\
             Open Windows Terminal (or conhost) and run: ytd\n\
             Or check the binary with: ytd --version\n\
             Or run diagnostics: ytd --doctor"
        );
    }

    color_eyre::install().wrap_err("install error handler")?;
    diag::breadcrumb("color_eyre ok");

    let config = Config::load();
    diag::breadcrumb("config loaded");
    let mut app = App::new(config);
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    app.set_action_tx(action_tx.clone());

    // Background: notify if a newer GitHub release exists (never blocks UI).
    updater::spawn_check(action_tx.clone());
    diag::breadcrumb("update check spawned");

    let mut tui = Tui::new().wrap_err("init TUI")?;
    diag::breadcrumb("Tui::new ok");
    tui.enter().wrap_err(
        "enter TUI mode — on Windows, prefer Windows Terminal over legacy console.\n\
         Run: ytd --doctor",
    )?;
    diag::breadcrumb("tui.enter ok");

    // Keep the event handler alive for the whole session (do not prefix with `_` alone in a way that drops).
    let _event_handler = EventHandler::new(action_tx.clone(), 4.0, 30.0);
    diag::breadcrumb("EventHandler started");

    // Initial render
    tui.terminal()
        .draw(|frame| ui::draw(frame, &app))
        .wrap_err("initial draw")?;
    diag::breadcrumb("initial draw ok — entering loop");

    // Keep sender alive so the channel never closes while the UI runs.
    let _keep_tx = action_tx;

    loop {
        let Some(action) = action_rx.recv().await else {
            // Should not happen while _keep_tx lives; treat as fatal.
            bail!("event channel closed unexpectedly");
        };

        let is_quit_probe = matches!(&action, Action::Key(_));
        app.update(action).wrap_err("app update")?;

        if app.should_quit {
            if is_quit_probe {
                diag::breadcrumb("quit requested via key");
            } else {
                diag::breadcrumb("quit requested");
            }
            break;
        }

        tui.terminal()
            .draw(|frame| ui::draw(frame, &app))
            .wrap_err("draw frame")?;
    }

    let should_restart = app.should_restart;
    let restart_path = app.restart_path.clone();
    tui.exit().wrap_err("leave TUI")?;
    diag::breadcrumb("tui.exit ok");

    if should_restart {
        diag::breadcrumb("reexec restart");
        if let Err(e) = updater::reexec_self(restart_path) {
            eprintln!("error: could not restart: {e:#}");
            eprintln!("launch manually: ytd");
            std::process::exit(1);
        }
    }

    diag::breadcrumb("clean exit");
    Ok(())
}
