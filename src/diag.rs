//! Diagnostics for silent failures (especially Windows).
//!
//! Always writes a breadcrumb log so we can see what happened even when the
//! console shows nothing. `ytui-dl --doctor` runs a full self-check.

use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use color_eyre::eyre::{Result, WrapErr};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Directory for logs / install metadata (not the TOML config dir).
pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ytui-dl")
}

pub fn log_path() -> PathBuf {
    data_dir().join("last-run.log")
}

/// Append a timestamped line to last-run.log. Never panics; best-effort only.
pub fn breadcrumb(msg: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{ts}] {msg}\n");
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
    // Also mirror to stderr when YTUI_DEBUG is set.
    if std::env::var_os("YTUI_DEBUG").is_some() {
        let _ = writeln!(io::stderr(), "[ytui-dl debug] {msg}");
        let _ = io::stderr().flush();
    }
}

/// Truncate the log at process start so each run is easy to read.
pub fn begin_run(args: &[String]) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "(unknown)".into());
    let header = format!(
        "=== ytui-dl {VERSION} start ===\nexe={exe}\nargs={args:?}\nos={}\narch={}\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
    );
    let _ = fs::write(&path, header);
    breadcrumb("begin_run");
}

fn yn(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// Interactive / CI-friendly self-check. Prints every step; never enters the full app loop.
pub fn run_doctor() -> Result<()> {
    begin_run(&["--doctor".into()]);
    breadcrumb("doctor start");

    println!("ytui-dl doctor {VERSION}");
    println!("========================");

    let exe = std::env::current_exe().wrap_err("current_exe")?;
    println!("executable : {}", exe.display());
    if let Ok(meta) = fs::metadata(&exe) {
        println!("size       : {} bytes", meta.len());
    }
    println!("log file   : {}", log_path().display());
    println!();

    let stdin_tty = io::stdin().is_terminal();
    let stdout_tty = io::stdout().is_terminal();
    let stderr_tty = io::stderr().is_terminal();
    println!("stdin  is terminal : {}", yn(stdin_tty));
    println!("stdout is terminal : {}", yn(stdout_tty));
    println!("stderr is terminal : {}", yn(stderr_tty));
    breadcrumb(&format!(
        "tty stdin={} stdout={} stderr={}",
        stdin_tty, stdout_tty, stderr_tty
    ));

    if !stdout_tty {
        println!();
        println!("WARN: stdout is not a TTY — the interactive TUI will refuse to start.");
        println!("      Open Windows Terminal / conhost and run: ytui-dl");
        breadcrumb("doctor: stdout not a tty");
    }

    println!();
    println!("Checking PATH tools…");
    for tool in ["yt-dlp", "ffmpeg"] {
        match which::which(tool) {
            Ok(p) => {
                println!("  {tool:8} : OK ({})", p.display());
                breadcrumb(&format!("tool {tool} ok"));
            }
            Err(_) => {
                println!("  {tool:8} : MISSING (optional for doctor; needed for downloads)");
                breadcrumb(&format!("tool {tool} missing"));
            }
        }
    }

    println!();
    if !stdout_tty {
        println!("Skipping raw-mode / alternate-screen test (no TTY).");
        println!();
        println!("Doctor finished with warnings.");
        breadcrumb("doctor done (no tty)");
        return Ok(());
    }

    println!("Testing terminal modes (raw + alternate screen)…");
    io::stdout().flush().ok();

    match enable_raw_mode() {
        Ok(()) => {
            println!("  enable_raw_mode     : OK");
            breadcrumb("enable_raw_mode ok");
        }
        Err(e) => {
            println!("  enable_raw_mode     : FAIL — {e}");
            breadcrumb(&format!("enable_raw_mode fail: {e}"));
            println!();
            println!("This is a common Windows console issue.");
            println!("Prefer Windows Terminal (not legacy ConHost alone).");
            return Ok(());
        }
    }

    let alt_result = {
        let mut out = io::stdout();
        out.execute(EnterAlternateScreen).map(|_| ())
    };
    match alt_result {
        Ok(()) => {
            // Visible marker on the alternate buffer (user may see a flash).
            let _ = write!(
                io::stdout(),
                "\r\n  ytui-dl doctor: alternate screen OK — restoring…\r\n"
            );
            let _ = io::stdout().flush();
            std::thread::sleep(std::time::Duration::from_millis(400));
            println!("  EnterAlternateScreen: OK");
            breadcrumb("EnterAlternateScreen ok");
        }
        Err(e) => {
            println!("  EnterAlternateScreen: FAIL — {e}");
            breadcrumb(&format!("EnterAlternateScreen fail: {e}"));
        }
    }

    {
        let mut out = io::stdout();
        let _ = out.execute(LeaveAlternateScreen);
    }
    if is_raw_mode_enabled().unwrap_or(false) {
        let _ = disable_raw_mode();
    }
    println!("  restore terminal   : OK");
    breadcrumb("restore ok");

    println!();
    println!("Doctor finished successfully.");
    println!("If plain `ytui-dl` still shows nothing, paste the contents of:");
    println!("  {}", log_path().display());
    breadcrumb("doctor done ok");
    Ok(())
}
