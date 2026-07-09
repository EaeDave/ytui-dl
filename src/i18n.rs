use serde::{Deserialize, Serialize};

/// UI language. Default is English.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    #[default]
    En,
    #[serde(alias = "pt_br", alias = "pt")]
    PtBr,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::En, Self::PtBr];

    pub fn next(self) -> Self {
        match self {
            Self::En => Self::PtBr,
            Self::PtBr => Self::En,
        }
    }

    /// Label shown in the language switcher (always native name).
    pub fn native_label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::PtBr => "Português (BR)",
        }
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Self::En => &EN,
            Self::PtBr => &PT_BR,
        }
    }
}

/// Static UI copy for one language.
pub struct Strings {
    // Screens / chrome
    pub screen_home: &'static str,
    pub screen_preview: &'static str,
    pub screen_queue: &'static str,
    pub screen_settings: &'static str,
    pub screen_help: &'static str,
    pub queue_count: &'static str,
    /// Short header badge when update exists (prefix; version appended in UI).
    pub update_badge: &'static str,

    // Status bar hints
    pub hint_home: &'static str,
    pub hint_preview: &'static str,
    pub hint_queue: &'static str,
    pub hint_settings: &'static str,
    pub hint_help: &'static str,

    // Home
    pub url_title: &'static str,
    pub options_title: &'static str,
    pub mode_label: &'static str,
    pub quality_label: &'static str,
    pub audio_label: &'static str,
    pub output_label: &'static str,
    pub enter_search: &'static str,
    pub enter_search_focus: &'static str,
    pub mode_video: &'static str,
    pub mode_audio: &'static str,
    pub quality_best: &'static str,
    pub quality_worst: &'static str,
    pub audio_best: &'static str,
    pub guide_title: &'static str,
    pub how_to: &'static str,
    pub how_step1: &'static str,
    pub how_step2: &'static str,
    pub how_step3: &'static str,
    pub ytdlp_ok: &'static str,
    pub ffmpeg_ok: &'static str,
    pub mode_hint: &'static str,
    pub quality_hint: &'static str,

    // Preview
    pub preview_empty: &'static str,
    pub video_info_title: &'static str,
    pub field_title: &'static str,
    pub field_channel: &'static str,
    pub field_duration: &'static str,
    pub field_id: &'static str,
    pub field_url: &'static str,
    pub field_mode: &'static str,
    pub field_quality: &'static str,
    pub field_format: &'static str,
    pub field_folder: &'static str,
    pub enter_download: &'static str,
    pub download_block: &'static str,
    pub quality_shortcuts: &'static str,
    pub reference_title: &'static str,

    // Queue
    pub queue_title: &'static str,
    pub queue_empty: &'static str,
    pub progress_title: &'static str,
    pub progress_selected: &'static str,
    pub status_queued: &'static str,
    pub status_downloading: &'static str,
    pub status_done: &'static str,
    pub status_failed: &'static str,
    pub status_cancelled: &'static str,

    // Settings
    pub settings_title: &'static str,
    pub settings_output: &'static str,
    pub settings_template: &'static str,
    pub settings_language: &'static str,
    pub settings_auto_open: &'static str,
    pub settings_on: &'static str,
    pub settings_off: &'static str,
    pub settings_tips: &'static str,
    pub settings_tip_template: &'static str,
    pub settings_tip_defaults: &'static str,
    pub settings_tip_file: &'static str,
    pub settings_tip_language: &'static str,
    pub settings_tip_auto_open: &'static str,
    pub settings_keys: &'static str,

    // Help
    pub help_title: &'static str,
    pub help_shortcuts: &'static str,
    pub help_nav: &'static str,
    pub help_home: &'static str,
    pub help_queue: &'static str,
    pub help_settings: &'static str,
    pub help_this: &'static str,
    pub help_esc: &'static str,
    pub help_quit: &'static str,
    pub help_download: &'static str,
    pub help_enter: &'static str,
    pub help_va: &'static str,
    pub help_m: &'static str,
    pub help_quality: &'static str,
    pub help_cancel: &'static str,
    pub help_open: &'static str,
    pub help_queue_section: &'static str,
    pub help_jk: &'static str,
    pub help_clear: &'static str,
    pub help_update: &'static str,
    pub help_footer: &'static str,

    // Update modal
    pub update_modal_title: &'static str,
    pub update_modal_confirm_keys: &'static str,
    pub update_modal_note: &'static str,
    pub update_modal_working: &'static str,
    pub update_modal_working_hint: &'static str,
    pub update_modal_done: &'static str,
    pub update_modal_restart_hint: &'static str,
    pub update_modal_quit_hint: &'static str,

    // Status / messages (static parts)
    pub status_paste_url: &'static str,
    pub status_up_to_date: &'static str,
    pub status_update_starting: &'static str,
    pub status_update_wait: &'static str,
    pub status_update_linux_only: &'static str,
    pub status_ffmpeg_missing: &'static str,
    pub status_download_started: &'static str,
    pub status_download_cancelled: &'static str,
    pub status_already_fetching: &'static str,
    pub status_ytdlp_unavailable: &'static str,
    pub status_need_url: &'static str,
    pub status_fetching: &'static str,
    pub status_no_video: &'static str,
    pub status_cancelling: &'static str,
    pub status_cancel_pending: &'static str,
    pub status_queue_item_cancelled: &'static str,
    pub status_no_active: &'static str,
    pub status_press_q: &'static str,
    pub status_active_downloads: &'static str,
    pub status_ctrl_c_hint: &'static str,
    pub status_output_empty: &'static str,
    pub status_template_empty: &'static str,
    pub status_settings_saved: &'static str,
    pub status_history_cleared: &'static str,
    pub status_term_read_error: &'static str,
}

pub static EN: Strings = Strings {
    screen_home: "Home",
    screen_preview: "Preview",
    screen_queue: "Queue",
    screen_settings: "Settings",
    screen_help: "Help",
    queue_count: "queue",
    update_badge: "update",

    hint_home: "Tab focus  Enter fetch  v/a mode  1-5 quality  f queue  u update  s settings  ? help  q quit",
    hint_preview: "Enter download  m mode  1-5 quality  Esc back  f queue  u update  ? help",
    hint_queue: "j/k navigate  p cancel  c clear finished  o folder  u update  Esc back",
    hint_settings: "Tab field  ←/→ language  Enter save  Esc cancel",
    hint_help: "Esc / ? / q close",

    url_title: "YouTube URL",
    options_title: "Options",
    mode_label: "Mode:      ",
    quality_label: "Quality:   ",
    audio_label: "Audio:     ",
    output_label: "Output:    ",
    enter_search: "  Enter to fetch video info",
    enter_search_focus: "  ▶  Enter to fetch / download",
    mode_video: "Video",
    mode_audio: "Audio",
    quality_best: "Best",
    quality_worst: "Worst",
    audio_best: "Best",
    guide_title: "Quick guide",
    how_to: "How to use",
    how_step1: "1. Paste a URL (terminal paste / Ctrl+Shift+V)",
    how_step2: "2. Pick Video or Audio and a quality preset",
    how_step3: "3. Enter → preview → Enter again to download",
    ytdlp_ok: "✓ yt-dlp detected",
    ffmpeg_ok: "✓ ffmpeg detected",
    mode_hint: "   (v/a or ←/→)",
    quality_hint: "   (1-5)",

    preview_empty: "No video loaded. Go home and fetch a URL.",
    video_info_title: "Video info",
    field_title: "Title:    ",
    field_channel: "Channel:  ",
    field_duration: "Duration: ",
    field_id: "ID:       ",
    field_url: "URL:      ",
    field_mode: "Mode:      ",
    field_quality: "Quality:   ",
    field_format: "Format:    ",
    field_folder: "Folder:    ",
    enter_download: "  Enter  →  add to queue and download",
    download_block: "Download",
    quality_shortcuts: "Quality shortcuts",
    reference_title: "Reference",

    queue_title: "Download queue",
    queue_empty: "Queue is empty. Fetch a video on the home screen and confirm download.",
    progress_title: "Progress",
    progress_selected: "Selected item progress",
    status_queued: "Queued",
    status_downloading: "Downloading",
    status_done: "Done",
    status_failed: "Failed",
    status_cancelled: "Cancelled",

    settings_title: "Settings",
    settings_output: "Output folder",
    settings_template: "Filename template (yt-dlp)",
    settings_language: "Language",
    settings_auto_open: "Auto-open after download",
    settings_on: "On",
    settings_off: "Off",
    settings_tips: "Tips",
    settings_tip_template: "• Template uses yt-dlp placeholders: %(title)s %(id)s %(ext)s …",
    settings_tip_defaults: "• Current mode/quality defaults are also saved on Enter",
    settings_tip_file: "• File: ~/.config/ytui-dl/config.toml",
    settings_tip_language: "• Language applies immediately and is saved with the rest",
    settings_tip_auto_open: "• Auto-open uses the system player (xdg-open / open / explorer)",
    settings_keys: "Enter = save   Esc = cancel   Tab = field   ←/→ = toggle",

    help_title: "Help",
    help_shortcuts: "Shortcuts",
    help_nav: "Navigation",
    help_home: "Home screen",
    help_queue: "Download queue",
    help_settings: "Settings",
    help_this: "This help",
    help_esc: "Back / close",
    help_quit: "Quit (Q forces quit with active download)",
    help_download: "Download",
    help_enter: "Fetch metadata / confirm download",
    help_va: "Video / audio mode",
    help_m: "Toggle mode (preview)",
    help_quality: "Quality presets (Best…Worst)",
    help_cancel: "Cancel active download",
    help_open: "Open output folder",
    help_queue_section: "Queue",
    help_jk: "Navigate items",
    help_clear: "Clear finished items",
    help_update: "Update to latest release (confirm, then install)",
    help_footer: "ytui-dl  ·  powered by yt-dlp  ·  Ratatui",

    update_modal_title: "Update available",
    update_modal_confirm_keys: "  Enter / y = install now    Esc / n = cancel  ",
    update_modal_note: "Downloads from GitHub Releases, verifies SHA256, replaces the binary.",
    update_modal_working: "Updating…",
    update_modal_working_hint: "Please wait — do not close the terminal.",
    update_modal_done: "Update installed",
    update_modal_restart_hint: "  Enter / R = restart ytui-dl now  ",
    update_modal_quit_hint: "Esc / q = stay on this session (restart later)",

    status_paste_url: "Paste a YouTube URL and press Enter",
    status_up_to_date: "You are on the latest release",
    status_update_starting: "Starting update…",
    status_update_wait: "Update in progress — please wait…",
    status_update_linux_only: "In-app update is Linux-only for now; use the install script on other OSes",
    status_ffmpeg_missing: "ffmpeg not found — video/audio merge and conversion may fail",
    status_download_started: "Download started…",
    status_download_cancelled: "Download cancelled",
    status_already_fetching: "Already fetching metadata…",
    status_ytdlp_unavailable: "yt-dlp not available",
    status_need_url: "Enter a YouTube URL",
    status_fetching: "Fetching info…",
    status_no_video: "No video loaded",
    status_cancelling: "Cancelling download…",
    status_cancel_pending: "Cancellation already in progress…",
    status_queue_item_cancelled: "Item removed from queue",
    status_no_active: "No active download",
    status_press_q: "Press q to quit",
    status_active_downloads: "Active downloads. Press p to cancel or Q (shift) to force quit",
    status_ctrl_c_hint: "Active download — press q again to quit or p to cancel",
    status_output_empty: "Output folder cannot be empty",
    status_template_empty: "Filename template cannot be empty",
    status_settings_saved: "Settings saved",
    status_history_cleared: "History cleared (finished items removed)",
    status_term_read_error: "terminal read error",
};

pub static PT_BR: Strings = Strings {
    screen_home: "Início",
    screen_preview: "Preview",
    screen_queue: "Fila",
    screen_settings: "Config",
    screen_help: "Ajuda",
    queue_count: "fila",
    update_badge: "atualização",

    hint_home: "Tab foco  Enter buscar  v/a modo  1-5 qualidade  f fila  u update  s config  ? ajuda  q sair",
    hint_preview: "Enter baixar  m modo  1-5 qualidade  Esc voltar  f fila  u update  ? ajuda",
    hint_queue: "j/k navegar  p cancelar  c limpar finalizados  o pasta  u update  Esc voltar",
    hint_settings: "Tab campo  ←/→ idioma  Enter salvar  Esc cancelar",
    hint_help: "Esc / ? / q fechar",

    url_title: "URL do YouTube",
    options_title: "Opções",
    mode_label: "Modo:      ",
    quality_label: "Qualidade: ",
    audio_label: "Áudio:     ",
    output_label: "Saída:     ",
    enter_search: "  Enter para buscar informações do vídeo",
    enter_search_focus: "  ▶  Enter para buscar / baixar",
    mode_video: "Vídeo",
    mode_audio: "Áudio",
    quality_best: "Melhor",
    quality_worst: "Pior",
    audio_best: "Melhor",
    guide_title: "Guia rápido",
    how_to: "Como usar",
    how_step1: "1. Cole a URL (paste do terminal / Ctrl+Shift+V)",
    how_step2: "2. Escolha Vídeo ou Áudio e a qualidade",
    how_step3: "3. Enter → preview → Enter de novo para baixar",
    ytdlp_ok: "✓ yt-dlp detectado",
    ffmpeg_ok: "✓ ffmpeg detectado",
    mode_hint: "   (v/a ou ←/→)",
    quality_hint: "   (1-5)",

    preview_empty: "Nenhum vídeo carregado. Volte ao início e busque uma URL.",
    video_info_title: "Informações do vídeo",
    field_title: "Título:   ",
    field_channel: "Canal:    ",
    field_duration: "Duração:  ",
    field_id: "ID:       ",
    field_url: "URL:      ",
    field_mode: "Modo:      ",
    field_quality: "Qualidade: ",
    field_format: "Formato:   ",
    field_folder: "Pasta:     ",
    enter_download: "  Enter  →  adicionar à fila e baixar",
    download_block: "Download",
    quality_shortcuts: "Atalhos de qualidade",
    reference_title: "Referência",

    queue_title: "Fila de downloads",
    queue_empty: "Fila vazia. Busque um vídeo na tela inicial e confirme o download.",
    progress_title: "Progresso",
    progress_selected: "Progresso do item selecionado",
    status_queued: "Na fila",
    status_downloading: "Baixando",
    status_done: "Concluído",
    status_failed: "Falhou",
    status_cancelled: "Cancelado",

    settings_title: "Configurações",
    settings_output: "Pasta de saída",
    settings_template: "Template do nome do arquivo (yt-dlp)",
    settings_language: "Idioma",
    settings_auto_open: "Abrir automaticamente após baixar",
    settings_on: "Sim",
    settings_off: "Não",
    settings_tips: "Dicas",
    settings_tip_template: "• Template usa placeholders do yt-dlp: %(title)s %(id)s %(ext)s …",
    settings_tip_defaults: "• Os defaults de modo/qualidade atuais também são salvos ao pressionar Enter",
    settings_tip_file: "• Arquivo: ~/.config/ytui-dl/config.toml",
    settings_tip_language: "• O idioma aplica na hora e é salvo junto com o resto",
    settings_tip_auto_open: "• Auto-open usa o player do sistema (xdg-open / open / explorer)",
    settings_keys: "Enter = salvar   Esc = cancelar   Tab = campo   ←/→ = alternar",

    help_title: "Ajuda",
    help_shortcuts: "Atalhos",
    help_nav: "Navegação",
    help_home: "Tela inicial",
    help_queue: "Fila de downloads",
    help_settings: "Configurações",
    help_this: "Esta ajuda",
    help_esc: "Voltar / fechar",
    help_quit: "Sair (Q força saída com download ativo)",
    help_download: "Download",
    help_enter: "Buscar metadata / confirmar download",
    help_va: "Modo vídeo / áudio",
    help_m: "Alternar modo (preview)",
    help_quality: "Presets de qualidade (Melhor…Pior)",
    help_cancel: "Cancelar download ativo",
    help_open: "Abrir pasta de saída",
    help_queue_section: "Fila",
    help_jk: "Navegar itens",
    help_clear: "Limpar itens finalizados",
    help_update: "Atualizar para a última release (confirma e instala)",
    help_footer: "ytui-dl  ·  powered by yt-dlp  ·  Ratatui",

    update_modal_title: "Atualização disponível",
    update_modal_confirm_keys: "  Enter / y = instalar agora    Esc / n = cancelar  ",
    update_modal_note: "Baixa do GitHub Releases, verifica SHA256 e substitui o binário.",
    update_modal_working: "Atualizando…",
    update_modal_working_hint: "Aguarde — não feche o terminal.",
    update_modal_done: "Atualização instalada",
    update_modal_restart_hint: "  Enter / R = reiniciar o ytui-dl agora  ",
    update_modal_quit_hint: "Esc / q = continuar nesta sessão (reinicie depois)",

    status_paste_url: "Cole uma URL do YouTube e pressione Enter",
    status_up_to_date: "Você já está na última release",
    status_update_starting: "Iniciando atualização…",
    status_update_wait: "Atualização em andamento — aguarde…",
    status_update_linux_only: "Update na TUI é só Linux por enquanto; use o script de install em outros SOs",
    status_ffmpeg_missing: "ffmpeg não encontrado — merge de vídeo/áudio e conversão podem falhar",
    status_download_started: "Download iniciado…",
    status_download_cancelled: "Download cancelado",
    status_already_fetching: "Já buscando metadata…",
    status_ytdlp_unavailable: "yt-dlp não disponível",
    status_need_url: "Informe uma URL do YouTube",
    status_fetching: "Buscando informações…",
    status_no_video: "Nenhum vídeo carregado",
    status_cancelling: "Cancelando download…",
    status_cancel_pending: "Cancelamento já em andamento…",
    status_queue_item_cancelled: "Item removido da fila",
    status_no_active: "Nenhum download ativo",
    status_press_q: "Pressione q para sair",
    status_active_downloads: "Há downloads ativos. Pressione p para cancelar ou Q (shift) para forçar saída",
    status_ctrl_c_hint: "Download ativo — pressione q de novo para sair ou p para cancelar",
    status_output_empty: "Pasta de saída não pode ser vazia",
    status_template_empty: "Template de nome não pode ser vazio",
    status_settings_saved: "Configurações salvas",
    status_history_cleared: "Histórico limpo (itens finalizados removidos)",
    status_term_read_error: "erro de leitura do terminal",
};

/// Format helpers that need dynamic values.
impl Language {
    pub fn msg_ready(self, title: &str) -> String {
        match self {
            Self::En => format!("Ready: {title}"),
            Self::PtBr => format!("Pronto: {title}"),
        }
    }

    pub fn msg_error(self, err: &str) -> String {
        match self {
            Self::En => format!("Error: {err}"),
            Self::PtBr => format!("Erro: {err}"),
        }
    }

    pub fn msg_done(self, title: &str) -> String {
        match self {
            Self::En => format!("Done: {title}"),
            Self::PtBr => format!("Concluído: {title}"),
        }
    }

    pub fn msg_failed(self, error: &str) -> String {
        match self {
            Self::En => format!("Failed: {error}"),
            Self::PtBr => format!("Falhou: {error}"),
        }
    }

    pub fn msg_queued(self, title: &str) -> String {
        match self {
            Self::En => format!("Queued: {title}"),
            Self::PtBr => format!("Adicionado à fila: {title}"),
        }
    }

    pub fn msg_saved_at(self, path: &str) -> String {
        match self {
            Self::En => format!("Saved to: {path}"),
            Self::PtBr => format!("Salvo em: {path}"),
        }
    }

    pub fn msg_save_error(self, e: &str) -> String {
        match self {
            Self::En => format!("Save error: {e}"),
            Self::PtBr => format!("Erro ao salvar: {e}"),
        }
    }

    pub fn msg_opening(self, path: &str) -> String {
        match self {
            Self::En => format!("Opening: {path}"),
            Self::PtBr => format!("Abrindo: {path}"),
        }
    }

    pub fn msg_open_failed(self, e: &str, path: &str) -> String {
        match self {
            Self::En => format!("Could not open: {e} (path: {path})"),
            Self::PtBr => format!("Não foi possível abrir: {e} (path: {path})"),
        }
    }

    pub fn msg_opened(self, path: &str) -> String {
        match self {
            Self::En => format!("Opened: {path}"),
            Self::PtBr => format!("Aberto: {path}"),
        }
    }

    pub fn msg_job_status(self, status: &str, title: &str) -> String {
        format!("{status} — {title}")
    }

    pub fn msg_update_available(self, version: &str) -> String {
        match self {
            Self::En => format!(
                "Update available: v{version}  ·  press u to install"
            ),
            Self::PtBr => format!(
                "Atualização disponível: v{version}  ·  pressione u para instalar"
            ),
        }
    }

    pub fn msg_update_confirm(self, version: &str) -> String {
        match self {
            Self::En => format!("Install v{version} now? Enter = yes, Esc = cancel"),
            Self::PtBr => format!("Instalar v{version} agora? Enter = sim, Esc = cancelar"),
        }
    }

    pub fn msg_update_confirm_body(self, version: &str) -> String {
        match self {
            Self::En => format!("A newer version is available: v{version}"),
            Self::PtBr => format!("Há uma versão mais nova: v{version}"),
        }
    }

    pub fn msg_update_done(self, version: &str) -> String {
        match self {
            Self::En => format!("Installed v{version} successfully"),
            Self::PtBr => format!("v{version} instalada com sucesso"),
        }
    }

    pub fn msg_update_failed(self, error: &str) -> String {
        match self {
            Self::En => format!("Update failed: {error}"),
            Self::PtBr => format!("Falha na atualização: {error}"),
        }
    }
}
