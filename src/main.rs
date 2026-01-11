use std::{
    error::Error,
    io,
    time::Duration,
};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use glob::glob;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, palette::tailwind},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use ratatui::{
    style::Stylize,
    widgets::{Clear, Gauge, ListState, Padding, Widget},
};

mod audyo;
use audyo::service::AudioService;

mod app;
use app::App;

use crate::downloader::{
    facade::YoutubeFacade,
};


mod downloader;
mod events;

const CUSTOM_LABEL_COLOR: Color = tailwind::CYAN.c800;
const GAUGE3_COLOR: Color = tailwind::BLUE.c800;

struct Buttons {
    states: ButtonStates,
}

enum ButtonStates {
    PlayOrPause,
    SpeedUp,
    SpeedDown,
    Forward,
    Backward,
}

#[derive(Debug)]
struct AudioFolder {
    path: String,
    files: Vec<String>,
}

impl AudioFolder {
    fn new(path: String) -> Self {
        Self {
            path: path,
            files: Vec::new(),
        }
    }
    fn load_mp3_file(&mut self) {
        let path = match glob(&self.path) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Invalid file path {}", &self.path);
                return;
            }
        };
        let mut files: Vec<_> = Vec::new();
        for entry in path {
            match entry {
                Ok(file) => {
                    let f = file.display().to_string();
                    files.push(f);
                }
                Err(e) => {
                    eprintln!("Glob error {}", e);
                    return;
                }
            };
        }
        self.files = files;
    }
}

#[derive(PartialEq)]
enum Focus {
    FolderList,
    Buttons,
    Popup,
}

impl<'a> App<'a> {
    fn next_folder(&mut self) {
        let i = match self.folder_state.selected() {
            Some(i) => (i + 1) % self.audio_folder.files.len(),
            None => 0,
        };
        self.folder_state.select(Some(i));
    }

    fn prev_folder(&mut self) {
        let i = match self.folder_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.audio_folder.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.folder_state.select(Some(i));
    }

    fn next_button(&mut self) {
        self.button_index = (self.button_index + 1) % self.buttons.len();
    }

    fn prev_button(&mut self) {
        self.button_index = if self.button_index == 0 {
            self.buttons.len() - 1
        } else {
            self.button_index - 1
        };
    }
}

impl App<'_> {
    fn render_main_page(&mut self, frame: &mut ratatui::Frame) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(frame.area());
        self.render_list_files(frame, horizontal[0]);

        let vertical = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(horizontal[1]);

        self.render_progress_bar(frame, vertical[1]);
        self.render_button(frame, vertical[2]);
        if self.focus == Focus::Popup {
            self.render_search_box(frame);
        }
    }

    fn render_list_files(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let folder_items: Vec<_> = self
            .audio_folder
            .files
            .iter()
            .map(|f| ListItem::new(f.clone()))
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Queue")
            .title_alignment(ratatui::layout::Alignment::Center);
        let hs = Style::default().fg(Color::Black).bg(Color::Green);

        let folder_list = List::new(folder_items)
            .block(block)
            .highlight_style(hs)
            .highlight_symbol(" >");
        frame.render_stateful_widget(folder_list, area, &mut self.folder_state);
    }

    fn render_button(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let button_chunks = Layout::horizontal([Constraint::Percentage(20); 6]).split(area);

        for (i, button) in self.buttons.iter().enumerate() {
            let is_selected = self.focus == Focus::Buttons && self.button_index == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1));
            let inner = block.inner(button_chunks[i]);
            let vertical = Layout::vertical([
                Constraint::Percentage(40),
                Constraint::Length(1),
                Constraint::Percentage(40),
            ])
            .split(inner);

            let p = Paragraph::new(*button)
                .style(style)
                .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(p, vertical[1]);
            frame.render_widget(block, button_chunks[i]);
        }
    }
    fn render_progress_bar(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let elapsed_time = formart_duration(self.audio_service.get_current_position());
        let total = formart_duration(Duration::new(self.audio_service.length as u64, 0));
        let ratio = if self.audio_service.length == 0 {
            0.0
        } else if (self.audio_service.get_current_position().as_secs_f64()
            / self.audio_service.length as f64)
            > 1.0
        {
            1.0
        } else {
            self.audio_service.get_current_position().as_secs_f64()
                / self.audio_service.length as f64
        };

        let span = Span::styled(
            format!("{}/{}", elapsed_time, total),
            Style::new().fg(CUSTOM_LABEL_COLOR),
        );
        let gauge = Gauge::default()
            .block(Block::default().title("Time").borders(Borders::ALL))
            .gauge_style(GAUGE3_COLOR)
            .ratio(ratio)
            .label(span);
        frame.render_widget(gauge, area);
    }

    fn render_search_box(&mut self, frame: &mut ratatui::Frame) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Download")
            .style(Style::default().fg(Color::Yellow));

        let paragraph = Paragraph::new("Test_popup")
            .style(Style::default().fg(Color::White))
            .block(block);

        let area = search_popup(frame.area(), 50, 25);
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}

fn search_popup(area: Rect, per_x: u16, per_y: u16) -> Rect {
    let vertical =
        Layout::vertical([Constraint::Percentage(per_y)]).flex(ratatui::layout::Flex::Center);
    let horizontal =
        Layout::horizontal([Constraint::Percentage(per_x)]).flex(ratatui::layout::Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn formart_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let ytb_facade = YoutubeFacade::new();
    // ytb_facade.download_audio("EdkLBDaxKd8").await?;

    let mut app = App::new(ytb_facade.output_dir.display().to_string());

    while !app.should_quit {
        terminal.draw(|f| {
            app.render_main_page(f);
        })?;

        app.handle_event()?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
