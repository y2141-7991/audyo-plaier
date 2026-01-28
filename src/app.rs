use std::time::Duration;

use ratatui::widgets::ListState;

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

        Self {
            folder_state,

            audio_service: AudioService::new(),
            audio_folder: audio_folder,
            buttons: vec!["-5s↩", "+↪5s", "◀◀", "▶⏸", "▶▶", ""],
            button_index: 0,
            focus: Focus::FolderList,
            tick_rate: Duration::from_millis(200),
            should_quit: false,
            text: TextInput::new(),
            ytb_facade: ytb_facade,
        }
    }
    pub fn load_folder(&mut self) {
        self.audio_folder.load_mp3_file();
    }
}
