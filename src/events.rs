use std::thread;

use crossterm::event::{self, Event as CEvent, KeyCode};
use tokio::runtime::Runtime;

use crate::{
    Focus,
    app::{App, SignalMessage},
    audyo::service::AudioEvent,
    downloader::{client::Result, facade::YoutubeFacade},
};

impl App<'_> {
    pub async fn handle_event(&mut self) -> Result<()> {
        self.poll_msg();
        if !event::poll(self.tick_rate)? {
            return Ok(());
        }

        let event = event::read()?;

        match event {
            CEvent::Key(key_event) => match key_event.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Tab => {
                    self.focus = if self.focus == Focus::Buttons {
                        Focus::FolderList
                    } else {
                        Focus::Buttons
                    }
                }
                KeyCode::Char('s') => {
                    if self.focus == Focus::Popup {
                        self.focus = Focus::FolderList;
                        self.text.clear();
                    } else {
                        self.focus = Focus::Popup
                    }
                }
                KeyCode::Enter if self.focus == Focus::Popup => {
                    self.focus = Focus::FolderList;
                    let video_id = self
                        .ytb_facade
                        .extract_video_id_from_url(&self.text.content);
                    if let Some(video_id) = video_id {
                        self.spawn_task_download(video_id);
                    }
                    self.text.clear();
                }
                KeyCode::Char('r') => {
                    self.load_folder();
                }

                KeyCode::Char('j') | KeyCode::Down => {
                    if self.focus == Focus::FolderList {
                        self.next_folder();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.focus == Focus::FolderList {
                        self.prev_folder();
                    }
                }

                KeyCode::Char('h') | KeyCode::Left => {
                    if self.focus == Focus::Buttons {
                        self.prev_button();
                    }
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    if self.focus == Focus::Buttons {
                        self.next_button();
                    }
                }
                KeyCode::Char(' ') => {
                    if self.focus == Focus::Buttons {
                        if let Some(i) = self.folder_state.selected() {
                            match self.button_index {
                                3 => {
                                    if self.audio_service.audio_event == AudioEvent::Play {
                                        if self.audio_service.current_audio
                                            != Some(self.audio_folder.files[i].clone())
                                        {
                                            self.audio_service
                                                .play(self.audio_folder.files[i].clone());
                                        } else {
                                            self.audio_service.audio_event = AudioEvent::Pause;
                                            self.audio_service.pause();
                                        }
                                    } else {
                                        self.audio_service.audio_event = AudioEvent::Play;
                                        self.audio_service.play(self.audio_folder.files[i].clone());
                                        self.folder_state.select(Some(i));
                                    }
                                }
                                4 => self.audio_service.speed_up(),
                                2 => self.audio_service.speed_down(),
                                1 => self.audio_service.seek_forward(),
                                0 => self.audio_service.seek_backward(),
                                5 => self.toggle_mode(),
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            },
            CEvent::Paste(pasted) if self.focus == Focus::Popup => {
                self.text.content.push_str(&pasted);
            }

            _ => {}
        }

        Ok(())
    }
}

impl App<'_> {
    fn spawn_task_download(&mut self, video_id: String) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let ytb_facade = YoutubeFacade::new();
                match ytb_facade.download_audio(&video_id).await {
                    Ok(()) => {
                        let _ = tx.send(SignalMessage::Downloaded);
                    }
                    Err(_) => {}
                }
            });
        });
    }
}
