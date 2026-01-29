use std::{fs::File, io::BufReader, time::Duration};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct AudioService {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    pub audio_event: AudioEvent,
    speed: f32,
    pub length: usize,
    pub current_audio: Option<String>,
    loop_single: bool,
    loop_playlist: bool,
    playlist: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum AudioEvent {
    Play,
    #[default]
    Pause,
}

impl AudioService {
    pub fn new() -> Self {
        let (_stream, _stream_handle) =
            OutputStream::try_default().expect("Can not init OutputStream");
        let sink = Sink::try_new(&_stream_handle).expect("Can not init Sink and PlayError");
        Self {
            _stream,
            _stream_handle,
            sink,
            audio_event: AudioEvent::default(),
            speed: 1.0,
            length: 1,
            current_audio: None,
            loop_playlist: false,
            loop_single: true,
            playlist: Vec::new(),
        }
    }
    pub fn play(&mut self, f: String) {
        if let Some(cur) = &self.current_audio {
            if f != *cur {
                self.stop();
                self.sink =
                    Sink::try_new(&self._stream_handle).expect("Can not init Sink and PlayError");
                self.current_audio = Some(f.clone());
                self._play(f);
            } else {
                self.current_audio = Some(f.clone());
                self._play(f);
            }
        } else {
            self.current_audio = Some(f.clone());
            self._play(f);
        }
    }
    fn _play(&mut self, f: String) {
        let file = File::open(f).expect("Can not file this file");
        let buf_reader = BufReader::new(file);
        let source = Decoder::new(buf_reader).expect("Decoder Error");
        self.length = if let Some(d) = source.total_duration() {
            d.as_secs() as usize
        } else {
            0
        };
        if self.loop_single {
            self.sink.append(source.repeat_infinite());
        } else {
            self.sink.append(source);
        }
        self.sink.play();
    }
    fn stop(&mut self) {
        self.sink.stop();
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
            current = Duration::from_secs(self.length as u64);
        } else {
            current += Duration::from_secs(5);
        }
        let _ = self.sink.try_seek(current);
    }
    pub fn seek_backward(&mut self) {
        let mut current = self.sink.get_pos();
        if current.as_secs() < 5 {
            current = Duration::from_secs(0)
        } else {
            current -= Duration::from_secs(5);
        }
        let _ = self.sink.try_seek(current);
    }
    pub fn get_current_position(&self) -> Duration {
        Duration::from_secs(self.sink.get_pos().as_secs() % (self.length as u64))
    }
}
