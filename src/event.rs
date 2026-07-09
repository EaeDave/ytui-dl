use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::action::Action;

/// Background task that translates terminal events into [`Action`]s.
pub struct EventHandler {
    _handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tx: mpsc::UnboundedSender<Action>, tick_rate: f64, frame_rate: f64) -> Self {
        let handle = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = interval(Duration::from_secs_f64(1.0 / tick_rate));
            let mut render_interval = interval(Duration::from_secs_f64(1.0 / frame_rate));
            // First ticks complete immediately — skip so we don't flood before first draw.
            tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            render_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                let tick = tick_interval.tick();
                let render = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = tick => {
                        if tx.send(Action::Tick).is_err() {
                            break;
                        }
                    }
                    _ = render => {
                        if tx.send(Action::Render).is_err() {
                            break;
                        }
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                if handle_crossterm(evt, &tx).is_err() {
                                    break;
                                }
                            }
                            Some(Err(_)) => {
                                let _ = tx.send(Action::Status(
                                    crate::i18n::Language::En
                                        .strings()
                                        .status_term_read_error
                                        .into(),
                                ));
                            }
                            // Do NOT tear down the whole app if the event stream ends —
                            // keep ticks/renders so the TUI stays alive (important on Windows).
                            None => {
                                // Replace dead stream after a short pause.
                                tokio::time::sleep(Duration::from_millis(50)).await;
                                reader = EventStream::new();
                            }
                        }
                    }
                }
            }
        });

        Self { _handle: handle }
    }
}

fn handle_crossterm(evt: CrosstermEvent, tx: &mpsc::UnboundedSender<Action>) -> Result<()> {
    match evt {
        // Windows emits Press and Release; some setups only expose the default kind.
        CrosstermEvent::Key(key)
            if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat =>
        {
            let _ = tx.send(Action::Key(key));
        }
        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Release => {
            // ignore releases
        }
        CrosstermEvent::Key(key) => {
            // Fallback for backends that don't set kind correctly.
            let _ = tx.send(Action::Key(key));
        }
        CrosstermEvent::Paste(text) => {
            let _ = tx.send(Action::Paste(text));
        }
        CrosstermEvent::Resize(w, h) => {
            let _ = tx.send(Action::Resize(w, h));
        }
        _ => {}
    }
    Ok(())
}
