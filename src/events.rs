use crossterm::event::{self, Event as CEvent, KeyCode};

use crate::{Focus, app::App, audyo::service::AudioEvent};

impl App<'_> {
    pub fn handle_event(&mut self) -> Result<(), std::io::Error> {
        if event::poll(self.tick_rate)? {
            if let CEvent::Key(key_event) = event::read()? {
                match key_event.code {
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
                            
                            self.focus = Focus::FolderList
                        } else {
                            self.focus = Focus::Popup
                        }
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
                                match self.buttons[self.button_index] {
                                    "▶⏸" => {
                                        if self.audio_service.audio_event == AudioEvent::Play {
                                            self.audio_service.audio_event = AudioEvent::Pause;
                                            self.audio_service.pause();
                                        } else {
                                            self.audio_service.audio_event = AudioEvent::Play;
                                            self.audio_service
                                                .play(self.audio_folder.files[i].clone())
                                        }
                                    }
                                    "▶▶" => self.audio_service.speed_up(),
                                    "◀◀" => self.audio_service.speed_down(),
                                    "+↪5s" => self.audio_service.seek_forward(),
                                    "-5s↩" => self.audio_service.seek_backward(),
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            else if self.focus == Focus::Popup {
                match event::read()? {
                    CEvent::Paste(pasted) => {  self.text += &pasted; println!("{}", pasted) }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
