// donut.rs

use ratatui::{prelude::*, widgets::Widget};

pub struct Donut {
    a: f32,
    b: f32,
}

impl Donut {
    pub fn new() -> Self {
        Self { a: 0.0, b: 0.0 }
    }

    pub fn tick(&mut self) {
        self.a += 0.28;
        self.b += 0.12;
    }

    fn render_frame(&self, width: usize, height: usize) -> Vec<Vec<char>> {
        let mut output = vec![vec![' '; width]; height];
        let mut zbuffer = vec![vec![0.0f32; width]; height];

        let cx = width as f32 / 2.0;
        let cy = height as f32 / 2.0;

        let scale_x = (width as f32 / 80.0) * 20.0;
        let scale_y = (height as f32 / 22.0) * 10.0;

        let luminance = " .:-=+*#%@";

        let mut j = 0.0f32;
        while j < 6.28 {
            let mut i = 0.0f32;
            while i < 6.28 {
                let c = i.sin();
                let d = j.cos();
                let e = self.a.sin();
                let f = j.sin();
                let g = self.a.cos();
                let h = d + 2.0;
                let big_d = 1.0 / (c * h * e + f * g + 5.0);
                let l = i.cos();
                let m = self.b.cos();
                let n = self.b.sin();
                let t = c * h * g - f * e;

                let x = (cx + scale_x * big_d * (l * h * m - t * n)) as i32;
                let y = (cy + scale_y * big_d * (l * h * n + t * m)) as i32;

                let brightness = 8.0 * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n);

                if y > 0
                    && (y as usize) < height
                    && x > 0
                    && (x as usize) < width
                    && big_d > zbuffer[y as usize][x as usize]
                {
                    zbuffer[y as usize][x as usize] = big_d;
                    let idx = if brightness > 0.0 {
                        (brightness as usize).min(luminance.len() - 1)
                    } else {
                        0
                    };
                    output[y as usize][x as usize] = luminance.chars().nth(idx).unwrap_or('.');
                }

                i += 0.02;
            }
            j += 0.07;
        }

        output
    }
}

impl Widget for &Donut {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 3 {
            return;
        }

        let width = area.width as usize;
        let height = area.height as usize;

        let frame = self.render_frame(width, height);

        for (y, row) in frame.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                if ch != ' ' {
                    let color = match ch {
                        '.' | ',' => Color::Rgb(64, 0, 128),  // Deep Purple
                        '-' | '~' => Color::Rgb(0, 64, 255),  // Blue
                        ':' | ';' => Color::Rgb(0, 200, 255), // Cyan
                        '=' | '!' => Color::Rgb(0, 255, 128), // Green
                        '*' | '#' => Color::Rgb(255, 200, 0), // Gold
                        '$' | '@' => Color::Rgb(255, 50, 0),  // Red
                        _ => Color::White,
                    };

                    buf.get_mut(area.x + x as u16, area.y + y as u16)
                        .set_char(ch)
                        .set_fg(color);
                }
            }
        }
    }
}
