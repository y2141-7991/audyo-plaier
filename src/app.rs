use ratatui::widgets::ListState;
use std::time::Duration;
use std::{sync::mpsc, time::Instant};

use crate::audyo::service::AudioEvent;
use crate::ui::donut::Donut;
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
    pub mute_sound: MuteSound,
    pub last_toggle_volume: Instant,
    pub tx: mpsc::Sender<SignalMessage>,
    rx: mpsc::Receiver<SignalMessage>,
    pub show_help: bool,
    pub sparkline_points: Signal<RandomSignal>,
    pub donut: Donut,
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
        let audio_service = AudioService::new();

        let mut vec_signal = RandomSignal::new(0, 100);
        let sparkline_points = vec_signal.by_ref().take(200).collect::<Vec<_>>();

        Self {
            folder_state,
            audio_service: audio_service,
            audio_folder: audio_folder,
            buttons: vec![],
            button_index: 0,
            focus: Focus::FolderList,
            tick_rate: Duration::from_millis(80),
            should_quit: false,
            text: TextInput::new(),
            ytb_facade: ytb_facade,
            loop_mode: LoopMode::Single,
            volume: Volume::Normal,
            mute_sound: MuteSound::Off,
            last_toggle_volume: Instant::now(),
            tx: tx,
            rx: rx,
            show_help: false,
            sparkline_points: Signal {
                source: vec_signal,
                points: sparkline_points,
                tick_rate: 2,
            },
            donut: Donut::new(),
        }
    }
    pub fn load_folder(&mut self) {
        self.audio_folder.load_mp3_file();
        self.audio_service.playlist = self.audio_folder.files.clone();
    }
    pub fn toggle_mode(&mut self) {
        self.loop_mode = self.loop_mode.next();
        self.audio_service.loop_mode = self.loop_mode;
    }
    pub fn poll_msg(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                SignalMessage::Downloaded => self.load_folder(),
                SignalMessage::UpdateIndex(index) => self.folder_state.select(Some(index)),
            }
        }
    }
    pub fn toggle_mute(&mut self) {
        if self.mute_sound == MuteSound::Off {
            self.audio_service.mute();
            self.mute_sound = self.mute_sound.on();
        } else {
            self.audio_service.unmute();
            self.mute_sound = self.mute_sound.off();
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
    pub fn audio_tick(&mut self) {
        if self.audio_service.audio_event == AudioEvent::Play {
            self.donut.tick();
        }
        match self.loop_mode {
            LoopMode::Single => {
                self.audio_service.single_mode();
            }
            LoopMode::Playlist | LoopMode::Shuffle => {
                let updated_idx = self.audio_service.playlist_mode();
                if let Some(idx) = updated_idx {
                    let _ = self.tx.send(SignalMessage::UpdateIndex(idx));
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug)]
pub enum Volume {
    Up,
    Down,
    Normal,
}

impl PartialEq for Volume {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Volume::Up, Volume::Up)
                | (Volume::Down, Volume::Down)
                | (Volume::Normal, Volume::Normal)
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
    pub fn text(&self) -> &'static str {
        match self {
            Self::Down => "⬇️",
            Self::Up => "⬆️",
            Self::Normal => "🔉",
        }
    }
}

pub enum MuteSound {
    On,
    Off,
}
impl MuteSound {
    pub fn on(&self) -> Self {
        Self::On
    }
    pub fn off(&self) -> Self {
        Self::Off
    }
    pub fn text(&self) -> &'static str {
        match self {
            Self::On => "🔇",
            Self::Off => "🔉",
        }
    }
}

impl PartialEq for MuteSound {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (Self::Off, Self::Off) | (Self::On, Self::On))
    }
}

pub enum SignalMessage {
    Downloaded,
    UpdateIndex(usize),
}

pub struct Signal<I: Iterator> {
    source: I,
    pub points: Vec<I::Item>,
    tick_rate: usize,
}
impl<I> Signal<I>
where
    I: Iterator,
{
    fn on_tick(&mut self) {
        self.points.drain(0..self.tick_rate);
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

#[derive(Clone)]
pub struct RandomSignal {
    distribution: rand::distr::Uniform<u64>,
    rng: rand::rngs::ThreadRng,
}
impl RandomSignal {
    pub fn new(lower: u64, upper: u64) -> Self {
        Self {
            distribution: rand::distr::Uniform::new(lower, upper).expect("invalid range"),
            rng: rand::rng(),
        }
    }
}
impl Iterator for RandomSignal {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        use rand::distr::Distribution;
        Some(self.distribution.sample(&mut self.rng))
    }
}
