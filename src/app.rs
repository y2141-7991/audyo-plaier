use ratatui::widgets::ListState;
use std::time::Duration;
use std::{sync::mpsc, time::Instant};

use crate::{AudioFolder, AudioService, Focus, downloader::facade::YoutubeFacade};

pub struct App<'a> {
    pub folder_state: ListState,

    pub audio_service: AudioService,
    pub audio_folder: AudioFolder,
    pub buttons: Vec<&'a str>,
    pub button_index: usize,
    pub focus: Focus,
    pub tick_rate: Duration,
    pub should_quit: bool,
    pub text: TextInput,

    pub ytb_facade: YoutubeFacade,
    pub loop_mode: LoopMode,
    pub volume: Volume,
    pub last_toggle_volume: Instant,
    pub tx: mpsc::Sender<SignalMessage>,
    rx: mpsc::Receiver<SignalMessage>,
    pub show_help: bool,
}

pub struct TextInput {
    pub content: String,
    cursor: usize,
}

impl TextInput {
    fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    fn insert(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += 1;
    }

    fn insert_str(&mut self, s: &str) {
        self.content.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    fn delete_back(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.content.remove(self.cursor);
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.content.len() {
            self.content.remove(self.cursor);
        }
    }

    fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.content.len());
    }

    fn move_start(&mut self) {
        self.cursor = 0;
    }

    fn move_end(&mut self) {
        self.cursor = self.content.len();
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
    }

    pub fn value(&self) -> &str {
        &self.content
    }
}

impl App<'_> {
    pub fn new() -> Self {
        let ytb_facade = YoutubeFacade::new();

        let audio_folder = AudioFolder::new().path(format!(
            "{}/*",
            ytb_facade.output_dir.display().to_string().clone()
        ));
        let mut folder_state = ListState::default();
        folder_state.select(Some(0));

        let (tx, rx) = mpsc::channel();

        Self {
            folder_state,
            audio_service: AudioService::new(),
            audio_folder: audio_folder,
            buttons: vec![],
            button_index: 0,
            focus: Focus::FolderList,
            tick_rate: Duration::from_millis(200),
            should_quit: false,
            text: TextInput::new(),
            ytb_facade: ytb_facade,
            loop_mode: LoopMode::Single,
            volume: Volume::Normal,
            last_toggle_volume: Instant::now(),
            tx: tx,
            rx: rx,
            show_help: false,
        }
    }
    pub fn load_folder(&mut self) {
        self.audio_folder.load_mp3_file();
    }
    pub fn toggle_mode(&mut self) {
        self.loop_mode = self.loop_mode.next();
    }
    pub fn poll_msg(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                SignalMessage::Downloaded => self.load_folder(),
            }
        }
    }
    pub fn toggle_mute(&mut self) {
        if self.volume == Volume::Normal {
            self.audio_service.mute();
            self.volume = self.volume.mute();
        } else {
            self.audio_service.unmute();
            self.volume = self.volume.normal();
        }
    }
    pub fn toggle_increase_vol(&mut self) {
        self.volume = self.volume.up();
        self.audio_service.increase_vol();
        self.last_toggle_volume = Instant::now();
    }
    pub fn toggle_decrease_vol(&mut self) {
        self.volume = self.volume.down();
        self.audio_service.decrease_vol();
        self.last_toggle_volume = Instant::now();
    }
}

pub enum LoopMode {
    Single,
    Playlist,
    Shuffle,
}

impl LoopMode {
    pub fn next(&self) -> Self {
        match self {
            Self::Single => Self::Playlist,
            Self::Playlist => Self::Shuffle,
            Self::Shuffle => Self::Single,
        }
    }
    pub fn text(&self) -> &'static str {
        match self {
            Self::Single => "🔂",
            Self::Playlist => "🔁",
            Self::Shuffle => "🔀",
        }
    }
}

pub enum Volume {
    Up,
    Down,
    Normal,
    Mute,
}

impl PartialEq for Volume {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Volume::Up, Volume::Up)
                | (Volume::Down, Volume::Down)
                | (Volume::Normal, Volume::Normal)
                | (Volume::Mute, Volume::Mute)
        )
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Volume {
    pub fn up(&self) -> Self {
        Self::Up
    }
    pub fn down(&self) -> Self {
        Self::Down
    }
    pub fn normal(&self) -> Self {
        Self::Normal
    }
    pub fn mute(&self) -> Self {
        Self::Mute
    }
    pub fn mute_text(&self) -> &'static str {
        "🔇"
    }
    pub fn text(&self) -> &'static str {
        match self {
            Self::Down => "⬇️",
            Self::Up => "⬆️",
            Self::Normal => "🔉",
            Self::Mute => "🔇",
        }
    }
}

pub enum SignalMessage {
    // Downloading,
    Downloaded,
    // Reloading
}
