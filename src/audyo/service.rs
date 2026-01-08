use std::{fs::File, time::Duration};

use rodio::{Decoder, OutputStream, Sink, Source};

pub struct AudioService {
    _stream: OutputStream,
    sink: Sink,
    pub audio_event: AudioEvent,
    speed: f32,
    pub length: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum AudioEvent {
    Play,
    #[default]
    Pause,
}

impl AudioService {
    pub fn new() -> Self {
        let (_stream, _hanlder) = OutputStream::try_default().expect("Can not init OutputStream");
        let sink = Sink::try_new(&_hanlder).expect("Can not init Sink and PlayError");
        Self {
            _stream,
            sink,
            audio_event: AudioEvent::default(),
            speed: 1.0,
            length: 0,
        }
    }
    pub fn play(&mut self, f: String) {
        let file = File::open(f).expect("Can not file this file");
        let source = Decoder::new(file).expect("Decoder Error");
        self.length = if let Some(d) = source.total_duration() {
            d.as_secs() as usize
        } else {
            0
        };
        self.sink.append(source);
        self.sink.play();
    }
    pub fn pause(&mut self) {
        self.sink.pause();
    }
    pub fn speed_up(&mut self) {
        self.speed += 0.25;
        self.sink.set_speed(self.speed);
    }
    pub fn speed_down(&mut self) {
        self.speed -= 0.25;
        self.sink.set_speed(self.speed);
    }
    pub fn seek_forward(&mut self) {
        let mut current = self.sink.get_pos();
        if self.length > 5 && (current.as_secs() as usize) >= (self.length - 5) {
            current = Duration::from_secs(self.length as u64)
        } else {
            current += Duration::from_secs(5);
        }
        self.sink.try_seek(current).expect("Can not seek more");
    }
    pub fn seek_backward(&mut self) {
        let mut current = self.sink.get_pos();
        if current.as_secs() < 5 {
            current = Duration::from_secs(0)
        } else {
            current -= Duration::from_secs(5);
        }
        self.sink.try_seek(current).expect("Can not seek more");
    }
    pub fn get_current_position(&self) -> Duration {
        self.sink.get_pos()
    }
}
