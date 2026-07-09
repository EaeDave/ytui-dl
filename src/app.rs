use std::path::PathBuf;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler as InputEventHandler;
use tui_input::Input;
use uuid::Uuid;

use crate::action::Action;
use crate::config::Config;
use crate::downloader::{
    build_output_template, fetch_video_info, new_path_tracker, start_download, watch_download,
    DownloadRequest, Tools,
};
use crate::i18n::{Language, Strings};
use crate::models::{
    AudioFormat, DownloadJob, Focus, JobStatus, MediaMode, OutputProfile, QualityPreset, Screen,
    VideoInfo,
};

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    pub previous_screen: Screen,
    pub focus: Focus,
    pub url_input: Input,
    pub settings_output_input: Input,
    pub settings_template_input: Input,
    pub mode: MediaMode,
    pub profile: OutputProfile,
    pub quality: QualityPreset,
    pub audio_format: AudioFormat,
    pub config: Config,
    pub jobs: Vec<DownloadJob>,
    pub queue_selected: usize,
    pub preview: Option<VideoInfo>,
    pub status_message: String,
    pub fetching: bool,
    pub tools: Option<Tools>,
    pub tools_warning: Option<String>,
    active_job_id: Option<Uuid>,
    /// Send `()` to cancel the active download worker.
    cancel_tx: Option<mpsc::UnboundedSender<()>>,
    action_tx: Option<mpsc::UnboundedSender<Action>>,
    tick_count: u64,
    /// Newer release version available (no leading `v`), if any.
    pub update_available: Option<String>,
    /// Waiting for Enter/Esc on the update confirm modal.
    pub update_confirm: bool,
    /// Download/install in progress.
    pub update_busy: bool,
    /// Install finished; waiting for R to restart.
    pub update_ready_restart: bool,
    /// After clean TUI exit, re-exec the binary.
    pub should_restart: bool,
    /// Preferred binary path for restart (avoids Linux `(deleted)` current_exe).
    pub restart_path: Option<PathBuf>,
}

impl App {
    pub fn lang(&self) -> Language {
        self.config.language
    }

    pub fn t(&self) -> &'static Strings {
        self.config.language.strings()
    }

    pub fn new(config: Config) -> Self {
        let mode = config.default_mode;
        let profile = config.default_profile;
        let quality = config.default_quality;
        let audio_format = config.default_audio_format;
        let t = config.language.strings();

        let (tools, tools_warning) = match Tools::detect() {
            Ok(t_tools) => {
                let warn = if !t_tools.has_ffmpeg() {
                    Some(t.status_ffmpeg_missing.into())
                } else {
                    None
                };
                (Some(t_tools), warn)
            }
            Err(e) => (None, Some(e.to_string())),
        };

        let status = tools_warning
            .clone()
            .unwrap_or_else(|| t.status_paste_url.into());

        let settings_output_input = Input::default().with_value(config.output_dir.display().to_string());
        let settings_template_input =
            Input::default().with_value(config.output_template.clone());

        Self {
            should_quit: false,
            screen: Screen::Home,
            previous_screen: Screen::Home,
            focus: Focus::UrlInput,
            url_input: Input::default(),
            settings_output_input,
            settings_template_input,
            mode,
            profile,
            quality,
            audio_format,
            config,
            jobs: Vec::new(),
            queue_selected: 0,
            preview: None,
            status_message: status,
            fetching: false,
            tools,
            tools_warning,
            active_job_id: None,
            cancel_tx: None,
            action_tx: None,
            tick_count: 0,
            update_available: None,
            update_confirm: false,
            update_busy: false,
            update_ready_restart: false,
            should_restart: false,
            restart_path: None,
        }
    }

    pub fn set_action_tx(&mut self, tx: mpsc::UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    pub fn has_active_download(&self) -> bool {
        self.active_job_id.is_some()
            || self
                .jobs
                .iter()
                .any(|j| matches!(j.status, JobStatus::Downloading | JobStatus::Queued))
    }

    pub fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Tick => {
                self.tick_count = self.tick_count.wrapping_add(1);
                self.try_start_next_job();
            }
            Action::Render | Action::Resize(_, _) => {}
            Action::Key(key) => self.on_key(key)?,
            Action::Paste(text) => self.on_paste(text),
            Action::MetadataReady(info) => {
                self.fetching = false;
                self.status_message = self.lang().msg_ready(&info.title);
                self.preview = Some(info);
                self.screen = Screen::Preview;
                self.focus = Focus::Confirm;
            }
            Action::MetadataFailed(err) => {
                self.fetching = false;
                self.status_message = self.lang().msg_error(&err);
                self.screen = Screen::Home;
                self.focus = Focus::UrlInput;
            }
            Action::DownloadStarted { job_id } => {
                if let Some(job) = self.find_job_mut(job_id) {
                    job.status = JobStatus::Downloading;
                    job.progress = 0.0;
                }
                self.status_message = self.t().status_download_started.into();
            }
            Action::DownloadProgress { job_id, update } => {
                if let Some(job) = self.find_job_mut(job_id) {
                    if let Some(p) = update.percent {
                        job.progress = p;
                    }
                    if update.speed.is_some() {
                        job.speed = update.speed;
                    }
                    if update.eta.is_some() {
                        job.eta = update.eta;
                    }
                    job.status = JobStatus::Downloading;
                }
            }
            Action::DownloadFinished {
                job_id,
                output_path,
            } => {
                let file_to_open = output_path.as_ref().filter(|p| p.is_file()).cloned();
                if let Some(job) = self.find_job_mut(job_id) {
                    job.status = JobStatus::Done;
                    job.progress = 100.0;
                    job.output_path = output_path;
                    job.speed = None;
                    job.eta = None;
                    let title = job.display_title().to_string();
                    self.status_message = self.lang().msg_done(&title);
                }
                if self.config.auto_open {
                    if let Some(path) = file_to_open {
                        match open_path(&path) {
                            Ok(()) => {
                                self.status_message =
                                    self.lang().msg_opened(&path.display().to_string());
                            }
                            Err(e) => {
                                self.status_message = self.lang().msg_open_failed(
                                    &e.to_string(),
                                    &path.display().to_string(),
                                );
                            }
                        }
                    }
                }
                self.clear_active_if(job_id);
                self.try_start_next_job();
            }
            Action::DownloadFailed { job_id, error } => {
                if let Some(job) = self.find_job_mut(job_id) {
                    job.status = JobStatus::Failed;
                    job.error = Some(error.clone());
                }
                self.status_message = self.lang().msg_failed(&error);
                self.clear_active_if(job_id);
                self.try_start_next_job();
            }
            Action::DownloadCancelled { job_id } => {
                if let Some(job) = self.find_job_mut(job_id) {
                    job.status = JobStatus::Cancelled;
                }
                self.status_message = self.t().status_download_cancelled.into();
                self.clear_active_if(job_id);
                self.try_start_next_job();
            }
            Action::UpdateAvailable { version } => {
                self.update_available = Some(version.clone());
                // Only surface if still on the default welcome/status (don't clobber errors).
                let t = self.t();
                if self.status_message == t.status_paste_url
                    || self
                        .tools_warning
                        .as_ref()
                        .is_some_and(|w| self.status_message == *w)
                {
                    self.status_message = self.lang().msg_update_available(&version);
                }
            }
            Action::UpdateProgress { message } => {
                self.status_message = format!("↑ {message}");
            }
            Action::UpdateSucceeded {
                version,
                install_path,
            } => {
                self.update_busy = false;
                self.update_confirm = false;
                self.update_ready_restart = true;
                self.update_available = None;
                self.restart_path = install_path
                    .filter(|p| p.is_file())
                    .or_else(|| crate::updater::resolve_restart_path().ok());
                self.status_message = self.lang().msg_update_done(&version);
            }
            Action::UpdateFailed { error } => {
                self.update_busy = false;
                self.update_confirm = false;
                self.update_ready_restart = false;
                self.status_message = self.lang().msg_update_failed(&error);
            }
            Action::Status(msg) => {
                self.status_message = msg;
            }
        }
        Ok(())
    }

    fn find_job_mut(&mut self, id: Uuid) -> Option<&mut DownloadJob> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    fn clear_active_if(&mut self, job_id: Uuid) {
        if self.active_job_id == Some(job_id) {
            self.active_job_id = None;
            self.cancel_tx = None;
        }
    }

    fn on_paste(&mut self, text: String) {
        let cleaned = text.trim().replace(['\n', '\r'], "");
        match self.screen {
            Screen::Home if self.focus == Focus::UrlInput => {
                let mut value = self.url_input.value().to_string();
                value.push_str(&cleaned);
                self.url_input = Input::default().with_value(value);
            }
            Screen::Settings if self.focus == Focus::SettingsOutput => {
                let mut value = self.settings_output_input.value().to_string();
                value.push_str(&cleaned);
                self.settings_output_input = Input::default().with_value(value);
            }
            Screen::Settings if self.focus == Focus::SettingsTemplate => {
                let mut value = self.settings_template_input.value().to_string();
                value.push_str(&cleaned);
                self.settings_template_input = Input::default().with_value(value);
            }
            _ => {}
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        // Global keys
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            if self.has_active_download() && !self.should_quit {
                self.status_message = self.t().status_ctrl_c_hint.into();
            }
            self.cancel_active();
            self.should_quit = true;
            return Ok(());
        }

        // Update modal / busy / restart prompts take over input.
        if self.update_busy {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                self.status_message = self.t().status_update_wait.into();
            }
            return Ok(());
        }

        if self.update_ready_restart {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Enter => {
                    self.should_restart = true;
                    self.should_quit = true;
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.update_ready_restart = false;
                    self.status_message = self.t().status_paste_url.into();
                }
                _ => {}
            }
            return Ok(());
        }

        if self.update_confirm {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.begin_update_install();
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.update_confirm = false;
                    if let Some(ver) = &self.update_available {
                        self.status_message = self.lang().msg_update_available(ver);
                    } else {
                        self.status_message = self.t().status_paste_url.into();
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        if self.screen == Screen::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')) {
                self.screen = self.previous_screen;
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') if !self.is_text_focus() => {
                if self.has_active_download() {
                    self.status_message = self.t().status_active_downloads.into();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('Q') if !self.is_text_focus() => {
                self.cancel_active();
                self.should_quit = true;
            }
            KeyCode::Char('?') if !self.is_text_focus() => {
                self.previous_screen = self.screen;
                self.screen = Screen::Help;
            }
            KeyCode::Char('f') if !self.is_text_focus() => {
                self.screen = Screen::Queue;
                self.focus = Focus::QueueList;
            }
            KeyCode::Char('s') if !self.is_text_focus() => {
                self.open_settings();
            }
            KeyCode::Char('h') if !self.is_text_focus() => {
                self.screen = Screen::Home;
                self.focus = Focus::UrlInput;
            }
            KeyCode::Char('p') if !self.is_text_focus() => {
                self.cancel_active();
            }
            KeyCode::Char('o') if !self.is_text_focus() => {
                self.open_output_dir();
            }
            KeyCode::Char('u') if !self.is_text_focus() => {
                self.prompt_update();
            }
            KeyCode::Esc => self.on_esc(),
            _ => match self.screen {
                Screen::Home => self.on_key_home(key),
                Screen::Preview => self.on_key_preview(key),
                Screen::Queue => self.on_key_queue(key),
                Screen::Settings => self.on_key_settings(key),
                Screen::Help => {}
            },
        }
        Ok(())
    }

    fn is_text_focus(&self) -> bool {
        matches!(
            self.focus,
            Focus::UrlInput | Focus::SettingsOutput | Focus::SettingsTemplate
        ) && matches!(self.screen, Screen::Home | Screen::Settings)
    }

    fn on_esc(&mut self) {
        match self.screen {
            Screen::Preview => {
                self.screen = Screen::Home;
                self.focus = Focus::UrlInput;
            }
            Screen::Queue | Screen::Settings | Screen::Help => {
                self.screen = Screen::Home;
                self.focus = Focus::UrlInput;
            }
            Screen::Home => {
                if self.is_text_focus() {
                    // clear is handled? just blur message
                    self.status_message = self.t().status_press_q.into();
                }
            }
        }
    }

    fn on_key_home(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => self.cycle_home_focus(false),
            KeyCode::BackTab => self.cycle_home_focus(true),
            KeyCode::Enter => match self.focus {
                Focus::UrlInput | Focus::Confirm => self.start_fetch_metadata(),
                Focus::Mode => {
                    self.mode = self.mode.toggle();
                }
                Focus::Profile => {
                    self.profile = self.profile.toggle();
                }
                Focus::Quality => {
                    self.quality = self.quality.next();
                }
                Focus::AudioFormat => {
                    self.audio_format = self.audio_format.next();
                }
                _ => self.start_fetch_metadata(),
            },
            KeyCode::Char('v') if !self.is_text_focus() || self.focus != Focus::UrlInput => {
                if self.focus != Focus::UrlInput {
                    self.mode = MediaMode::Video;
                } else {
                    self.forward_to_url_input(key);
                }
            }
            KeyCode::Char('a') if self.focus != Focus::UrlInput => {
                self.mode = MediaMode::Audio;
            }
            KeyCode::Char('w') if self.focus != Focus::UrlInput => {
                self.profile = OutputProfile::WhatsApp;
            }
            KeyCode::Char('b') if self.focus != Focus::UrlInput => {
                self.profile = OutputProfile::Best;
            }
            KeyCode::Char(c) if self.focus != Focus::UrlInput && c.is_ascii_digit() => {
                if let Some(q) = QualityPreset::from_digit(c) {
                    self.quality = q;
                }
            }
            KeyCode::Left | KeyCode::Right if self.focus == Focus::Mode => {
                self.mode = self.mode.toggle();
            }
            KeyCode::Left | KeyCode::Right if self.focus == Focus::Profile => {
                self.profile = self.profile.toggle();
            }
            KeyCode::Right if self.focus == Focus::Quality => {
                self.quality = self.quality.next();
            }
            KeyCode::Left if self.focus == Focus::Quality => {
                self.quality = self.quality.prev();
            }
            KeyCode::Right if self.focus == Focus::AudioFormat => {
                self.audio_format = self.audio_format.next();
            }
            KeyCode::Left if self.focus == Focus::AudioFormat => {
                self.audio_format = self.audio_format.prev();
            }
            _ if self.focus == Focus::UrlInput => {
                self.forward_to_url_input(key);
            }
            _ => {}
        }
    }

    fn forward_to_url_input(&mut self, key: KeyEvent) {
        self.url_input
            .handle_event(&crossterm::event::Event::Key(key));
    }

    fn cycle_home_focus(&mut self, reverse: bool) {
        let order = [
            Focus::UrlInput,
            Focus::Mode,
            Focus::Profile,
            Focus::Quality,
            Focus::AudioFormat,
            Focus::Confirm,
        ];
        let idx = order.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = if reverse {
            if idx == 0 {
                order.len() - 1
            } else {
                idx - 1
            }
        } else {
            (idx + 1) % order.len()
        };
        self.focus = order[next];
    }

    fn on_key_preview(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char('d') => self.enqueue_from_preview(),
            KeyCode::Char('v') => self.mode = MediaMode::Video,
            KeyCode::Char('a') => self.mode = MediaMode::Audio,
            KeyCode::Char('w') => self.profile = OutputProfile::WhatsApp,
            KeyCode::Char('b') => self.profile = OutputProfile::Best,
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(q) = QualityPreset::from_digit(c) {
                    self.quality = q;
                }
            }
            KeyCode::Tab | KeyCode::Right => {
                if self.mode == MediaMode::Video {
                    self.quality = self.quality.next();
                } else {
                    self.audio_format = self.audio_format.next();
                }
            }
            KeyCode::BackTab | KeyCode::Left => {
                if self.mode == MediaMode::Video {
                    self.quality = self.quality.prev();
                } else {
                    self.audio_format = self.audio_format.prev();
                }
            }
            KeyCode::Char('m') => self.mode = self.mode.toggle(),
            _ => {}
        }
    }

    fn on_key_queue(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.queue_selected > 0 {
                    self.queue_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.jobs.is_empty() {
                    self.queue_selected = (self.queue_selected + 1).min(self.jobs.len() - 1);
                }
            }
            KeyCode::Char('p') => self.cancel_active(),
            KeyCode::Enter => {
                if let Some(job) = self.jobs.get(self.queue_selected) {
                    if let Some(path) = job.output_path.clone() {
                        if path.is_file() {
                            match open_path(&path) {
                                Ok(()) => {
                                    self.status_message =
                                        self.lang().msg_opened(&path.display().to_string());
                                }
                                Err(e) => {
                                    self.status_message = self.lang().msg_open_failed(
                                        &e.to_string(),
                                        &path.display().to_string(),
                                    );
                                }
                            }
                        } else {
                            self.status_message =
                                self.lang().msg_saved_at(&path.display().to_string());
                        }
                    } else if let Some(err) = &job.error {
                        self.status_message = self.lang().msg_error(err);
                    } else {
                        let status = job.status.label(self.t());
                        self.status_message =
                            self.lang().msg_job_status(status, job.display_title());
                    }
                }
            }
            KeyCode::Char('c') => {
                // clear finished jobs
                self.jobs.retain(|j| !j.status.is_terminal());
                self.queue_selected = 0;
                self.status_message = self.t().status_history_cleared.into();
            }
            _ => {}
        }
    }

    fn on_key_settings(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::SettingsOutput => Focus::SettingsTemplate,
                    Focus::SettingsTemplate => Focus::SettingsLanguage,
                    Focus::SettingsLanguage => Focus::SettingsAutoOpen,
                    Focus::SettingsAutoOpen => Focus::SettingsOutput,
                    _ => Focus::SettingsOutput,
                };
            }
            KeyCode::BackTab => {
                self.focus = match self.focus {
                    Focus::SettingsOutput => Focus::SettingsAutoOpen,
                    Focus::SettingsTemplate => Focus::SettingsOutput,
                    Focus::SettingsLanguage => Focus::SettingsTemplate,
                    Focus::SettingsAutoOpen => Focus::SettingsLanguage,
                    _ => Focus::SettingsOutput,
                };
            }
            KeyCode::Enter => {
                if self.focus == Focus::SettingsLanguage {
                    self.config.language = self.config.language.next();
                    self.status_message = format!(
                        "{}: {}",
                        self.t().settings_language,
                        self.config.language.native_label()
                    );
                } else if self.focus == Focus::SettingsAutoOpen {
                    self.config.auto_open = !self.config.auto_open;
                } else {
                    self.apply_and_save_settings();
                }
            }
            KeyCode::Left | KeyCode::Right if self.focus == Focus::SettingsLanguage => {
                self.config.language = self.config.language.next();
            }
            KeyCode::Left | KeyCode::Right if self.focus == Focus::SettingsAutoOpen => {
                self.config.auto_open = !self.config.auto_open;
            }
            _ => {
                if matches!(
                    self.focus,
                    Focus::SettingsOutput | Focus::SettingsTemplate
                ) {
                    self.forward_settings_input(key);
                }
            }
        }
    }

    fn forward_settings_input(&mut self, key: KeyEvent) {
        let ev = crossterm::event::Event::Key(key);
        match self.focus {
            Focus::SettingsOutput => {
                self.settings_output_input.handle_event(&ev);
            }
            Focus::SettingsTemplate => {
                self.settings_template_input.handle_event(&ev);
            }
            _ => {}
        }
    }

    fn open_settings(&mut self) {
        self.settings_output_input =
            Input::default().with_value(self.config.output_dir.display().to_string());
        self.settings_template_input =
            Input::default().with_value(self.config.output_template.clone());
        self.screen = Screen::Settings;
        self.focus = Focus::SettingsOutput;
    }

    fn apply_and_save_settings(&mut self) {
        let out = self.settings_output_input.value().trim();
        let tmpl = self.settings_template_input.value().trim();
        if out.is_empty() {
            self.status_message = self.t().status_output_empty.into();
            return;
        }
        if tmpl.is_empty() {
            self.status_message = self.t().status_template_empty.into();
            return;
        }
        self.config.output_dir = PathBuf::from(out);
        self.config.output_template = tmpl.to_string();
        self.config.default_mode = self.mode;
        self.config.default_profile = self.profile;
        self.config.default_quality = self.quality;
        self.config.default_audio_format = self.audio_format;
        // language + auto_open already live on config
        match self.config.save() {
            Ok(()) => {
                self.status_message = self.t().status_settings_saved.into();
                self.screen = Screen::Home;
                self.focus = Focus::UrlInput;
            }
            Err(e) => {
                self.status_message = self.lang().msg_save_error(&e.to_string());
            }
        }
    }

    fn start_fetch_metadata(&mut self) {
        if self.fetching {
            self.status_message = self.t().status_already_fetching.into();
            return;
        }
        let Some(tools) = self.tools.clone() else {
            self.status_message = self
                .tools_warning
                .clone()
                .unwrap_or_else(|| self.t().status_ytdlp_unavailable.into());
            return;
        };
        let url = self.url_input.value().trim().to_string();
        if url.is_empty() {
            self.status_message = self.t().status_need_url.into();
            return;
        }
        let Some(tx) = self.action_tx.clone() else {
            return;
        };

        self.fetching = true;
        self.status_message = self.t().status_fetching.into();

        tokio::spawn(async move {
            match fetch_video_info(&tools, &url).await {
                Ok(info) => {
                    let _ = tx.send(Action::MetadataReady(info));
                }
                Err(e) => {
                    let _ = tx.send(Action::MetadataFailed(e.to_string()));
                }
            }
        });
    }

    fn enqueue_from_preview(&mut self) {
        let Some(preview) = self.preview.clone() else {
            self.status_message = self.t().status_no_video.into();
            return;
        };
        if self.tools.is_none() {
            self.status_message = self
                .tools_warning
                .clone()
                .unwrap_or_else(|| self.t().status_ytdlp_unavailable.into());
            return;
        }

        if self.profile == OutputProfile::WhatsApp && self.tools.as_ref().is_some_and(|t| !t.has_ffmpeg())
        {
            self.status_message = self.t().status_whatsapp_needs_ffmpeg.into();
            return;
        }

        let job = DownloadJob::new(
            preview.webpage_url.clone(),
            self.mode,
            self.profile,
            self.quality,
            self.audio_format,
            Some(preview.title.clone()),
        );
        self.jobs.push(job);
        self.queue_selected = self.jobs.len().saturating_sub(1);
        self.screen = Screen::Queue;
        self.focus = Focus::QueueList;
        self.status_message = self.lang().msg_queued(&preview.title);
        self.try_start_next_job();
    }

    fn try_start_next_job(&mut self) {
        if self.active_job_id.is_some() {
            return;
        }
        let Some(tools) = self.tools.clone() else {
            return;
        };
        let Some(tx) = self.action_tx.clone() else {
            return;
        };

        let next_idx = self.jobs.iter().position(|j| j.status == JobStatus::Queued);
        let Some(idx) = next_idx else {
            return;
        };

        let job = &self.jobs[idx];
        let job_id = job.id;
        let req = DownloadRequest {
            job_id,
            url: job.url.clone(),
            mode: job.mode,
            profile: job.profile,
            quality: job.quality,
            audio_format: job.audio_format,
            output_template: build_output_template(
                &self.config.output_dir,
                &self.config.output_template,
            ),
        };
        let output_dir = self.config.output_dir.clone();

        if let Some(job) = self.find_job_mut(job_id) {
            job.status = JobStatus::Downloading;
        }
        self.active_job_id = Some(job_id);

        let (cancel_tx, cancel_rx) = mpsc::unbounded_channel();
        self.cancel_tx = Some(cancel_tx);

        tokio::spawn(async move {
            let last_path = new_path_tracker();
            match start_download(&tools, req, tx.clone(), last_path.clone()).await {
                Ok(child) => {
                    watch_download(child, job_id, output_dir, last_path, tx, cancel_rx).await;
                }
                Err(e) => {
                    let _ = tx.send(Action::DownloadFailed {
                        job_id,
                        error: e.to_string(),
                    });
                }
            }
        });
    }

    fn cancel_active(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
            self.status_message = self.t().status_cancelling.into();
        } else if self.active_job_id.is_some() {
            self.status_message = self.t().status_cancel_pending.into();
        } else if let Some(job) = self.jobs.iter_mut().find(|j| j.status == JobStatus::Queued)
        {
            job.status = JobStatus::Cancelled;
            self.status_message = self.t().status_queue_item_cancelled.into();
        } else {
            self.status_message = self.t().status_no_active.into();
        }
    }

    fn open_output_dir(&mut self) {
        let dir = self.config.output_dir.clone();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.display().to_string();
        // xdg-open / open / explorer
        let result = open_path(&dir);
        self.status_message = match result {
            Ok(()) => self.lang().msg_opening(&path),
            Err(e) => self.lang().msg_open_failed(&e.to_string(), &path),
        };
    }

    pub fn spinner_frame(&self) -> &'static str {
        const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        FRAMES[(self.tick_count as usize / 2) % FRAMES.len()]
    }

    fn prompt_update(&mut self) {
        if self.update_busy {
            return;
        }
        if let Some(ver) = &self.update_available {
            let ver = ver.clone();
            self.update_confirm = true;
            self.status_message = self.lang().msg_update_confirm(&ver);
        } else {
            self.status_message = self.t().status_up_to_date.into();
        }
    }

    fn begin_update_install(&mut self) {
        let Some(tx) = self.action_tx.clone() else {
            return;
        };
        self.update_confirm = false;
        self.update_busy = true;
        self.status_message = self.t().status_update_starting.into();
        crate::updater::spawn_tui_update(tx);
    }
}

fn open_path(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // `start` opens files with the default app; for folders it opens Explorer.
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(path)
            .spawn()?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
        return Ok(());
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
        Ok(())
    }
}
