use std::{error::Error, io, time::Duration};

use crossterm::{
    event::EnableBracketedPaste,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use glob::glob;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, palette::tailwind},
    symbols::bar::NINE_LEVELS,
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
};
use ratatui::{
    text::Line,
    widgets::{Clear, Gauge, Padding},
};

mod audyo;
use audyo::service::AudioService;

mod app;
use app::App;

mod downloader;
mod events;

const CUSTOM_LABEL_COLOR: Color = tailwind::WHITE;
const GAUGE3_COLOR: Color = tailwind::GRAY.c800;

// struct Buttons {
//     states: ButtonStates,
// }

// enum ButtonStates {
//     PlayOrPause,
//     SpeedUp,
//     SpeedDown,
//     Forward,
//     Backward,
// }

#[derive(Debug, Clone)]
struct AudioFolder {
    path: String,
    files: Vec<String>,
}

impl AudioFolder {
    fn new() -> Self {
        Self {
            path: String::new(),
            files: Vec::new(),
        }
    }
    fn path(mut self, path: String) -> Self {
        self.path = path;
        self
    }
    fn load_mp3_file(&mut self) {
        let path = match glob(&self.path) {
            Ok(path) => path,
            Err(_) => {
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

#[derive(PartialEq, Debug)]
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

        self.render_waveform(frame, vertical[0]);
        self.render_progress_bar(frame, vertical[1]);
        self.render_button(frame, vertical[2]);
        if self.focus == Focus::Popup {
            self.render_search_popup(frame);
        }
        if self.show_help {
            self.render_help_popup(frame);
        }
    }
    fn render_waveform(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let sparkline = Sparkline::default()
            .block(Block::default())
            .style(Style::default().fg(Color::Green))
            .data(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 01, 3, 4, 5, 2, 6])
            .bar_set(NINE_LEVELS);
        frame.render_widget(sparkline, area);
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
        self.render_help_box(frame, area);
    }

    fn render_button(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        self.buttons = vec![
            self.mute_sound.text(),
            self.volume.text(),
            "⏮️",
            "⏯️",
            "⏭️",
            self.loop_mode.text(),
        ];
        let button_chunks = Layout::horizontal([Constraint::Percentage(20); 6]).split(area);

        for (i, button) in self.buttons.iter().enumerate() {
            let is_selected = self.focus == Focus::Buttons && self.button_index == i;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(tailwind::CYAN.c100)
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

    fn render_search_popup(&mut self, frame: &mut ratatui::Frame) {
        let area = _popup(frame.area(), 50, 25);

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Download")
            .style(Style::default().fg(Color::Yellow));

        let paragraph = Paragraph::new(self.text.value())
            .style(Style::default().fg(Color::White))
            .block(block);

        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
    fn render_help_popup(&mut self, frame: &mut ratatui::Frame) {
        let area = _popup(frame.area(), 25, 50);
        let help_lines = vec![
            Line::from(vec![Span::styled(
                "  NAVIGATION",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![
                Span::styled("    Tab    ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch focus"),
            ]),
            Line::from(vec![
                Span::styled("    j/↓    ", Style::default().fg(Color::Cyan)),
                Span::raw("Next track"),
            ]),
            Line::from(vec![
                Span::styled("    k/↑    ", Style::default().fg(Color::Cyan)),
                Span::raw("Previous track"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  PLAYBACK",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![
                Span::styled("    Space  ", Style::default().fg(Color::Cyan)),
                Span::raw("Activate button"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  DOWNLOAD",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![
                Span::styled("    s      ", Style::default().fg(Color::Cyan)),
                Span::raw("Open download"),
            ]),
            Line::from(vec![
                Span::styled("    Ctrl+V ", Style::default().fg(Color::Cyan)),
                Span::raw("Paste URL"),
            ]),
            Line::from(vec![
                Span::styled("    Enter  ", Style::default().fg(Color::Cyan)),
                Span::raw("Processing download"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  OTHER",
                Style::default().fg(Color::Yellow),
            )]),
            Line::from(vec![
                Span::styled("    r      ", Style::default().fg(Color::Cyan)),
                Span::raw("Reload folder"),
            ]),
            Line::from(vec![
                Span::styled("    q      ", Style::default().fg(Color::Cyan)),
                Span::raw("Quit"),
            ]),
            Line::from(vec![
                Span::styled("    /      ", Style::default().fg(Color::Cyan)),
                Span::raw("Close help"),
            ]),
        ];

        let help = Paragraph::new(help_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" 🎵 Help "),
        );
        frame.render_widget(Clear, area);
        frame.render_widget(help, area);
    }

    fn render_help_box(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let area = length_box(area, 10, 3);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        let paragraph = Paragraph::new("/: Help")
            .style(Style::default().fg(Color::White))
            .block(block)
            .centered();
        frame.render_widget(paragraph, area);
    }
}

fn _popup(area: Rect, per_x: u16, per_y: u16) -> Rect {
    let vertical =
        Layout::vertical([Constraint::Percentage(per_y)]).flex(ratatui::layout::Flex::Center);
    let horizontal =
        Layout::horizontal([Constraint::Percentage(per_x)]).flex(ratatui::layout::Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
fn length_box(area: Rect, len_x: u16, len_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(len_y)]).flex(ratatui::layout::Flex::End);
    let horizontal =
        Layout::horizontal([Constraint::Length(len_x)]).flex(ratatui::layout::Flex::End);
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
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend: CrosstermBackend<io::Stdout> = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    app.load_folder();
    while !app.should_quit {
        app.audio_service.tick();
        terminal.draw(|f| {
            app.render_main_page(f);
        })?;

        app.handle_event().await?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
