use std::time::Duration;

use ratatui::widgets::ListState;

use crate::{AudioFolder, AudioService, Focus};

pub struct App<'a> {
    pub folder_state: ListState,

    pub audio_service: AudioService,
    pub audio_folder: AudioFolder,
    pub buttons: Vec<&'a str>,
    pub button_index: usize,
    pub focus: Focus,
    pub tick_rate: Duration,
    pub should_quit: bool,
}

impl App<'_> {
    pub fn new(path: String) -> Self {
        let mut audio_folder = AudioFolder::new(format!("{}/*", path.clone()));
        audio_folder.load_mp3_file();

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
        }
    }
}
