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

use color_eyre::eyre::Result;
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
  ytui-dl --help       Print this help

Install / uninstall (script):
  curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
  curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --uninstall
"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
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
                if let Err(e) = updater::run_self_update(force).await {
                    eprintln!("error: {e:#}");
                    std::process::exit(1);
                }
                return Ok(());
            }
            other => {
                eprintln!("unknown argument: {other}");
                eprintln!("try: ytui-dl --help");
                std::process::exit(2);
            }
        }
    }

    color_eyre::install()?;

    let config = Config::load();
    let mut app = App::new(config);
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    app.set_action_tx(action_tx.clone());

    // Background: notify if a newer GitHub release exists (never blocks UI).
    updater::spawn_check(action_tx.clone());

    let mut tui = Tui::new()?;
    tui.enter()?;

    // Event pump: keyboard / tick / render
    let _events = EventHandler::new(action_tx.clone(), 4.0, 30.0);

    // Initial render
    tui.terminal().draw(|frame| ui::draw(frame, &app))?;

    loop {
        let Some(action) = action_rx.recv().await else {
            break;
        };

        app.update(action)?;

        if app.should_quit {
            break;
        }

        // Redraw after every action for snappy UX (event loop is already rate-limited by ticks)
        tui.terminal().draw(|frame| ui::draw(frame, &app))?;
    }

    let should_restart = app.should_restart;
    tui.exit()?;

    if should_restart {
        // Linux: re-exec the (updated) binary. Clean TUI teardown already done.
        if let Err(e) = updater::reexec_self() {
            eprintln!("error: could not restart: {e:#}");
            eprintln!("launch manually: ytui-dl");
            std::process::exit(1);
        }
    }

    Ok(())
}
