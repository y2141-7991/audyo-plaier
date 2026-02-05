use std::{fs::File, io::BufReader, time::Duration};

use rand::Rng;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use crate::app::LoopMode;

pub struct AudioService {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    pub audio_event: AudioEvent,
    speed: f32,
    pub length: usize,
    pub current_audio: Option<String>,
    pub current_playlist_index: usize,
    pub current_volume: f32,
    pub playlist: Vec<String>,
    pub loop_mode: LoopMode,
    pub waveform: WaveFormData,
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
        sink.pause();
        let cur_vol = sink.volume();
        Self {
            _stream,
            _stream_handle,
            sink,
            audio_event: AudioEvent::default(),
            speed: 1.0,
            length: 1,
            current_audio: None,
            current_volume: cur_vol,
            current_playlist_index: 0,
            playlist: Vec::new(),
            loop_mode: LoopMode::Single,
            waveform: WaveFormData {
                samples: Vec::new(),
                sample_rate: 0,
                durations: 0,
            },
        }
    }
    pub fn play(&mut self) {
        self.sink.play();
    }
    fn append_source_to_sink_from_file(&mut self, f: String) {
        let file = File::open(&f).expect("Can not file this file");
        let buf_reader = BufReader::new(file);
        let source = Decoder::new(buf_reader).expect("Decoder Error");
        self.length = if let Some(d) = source.total_duration() {
            d.as_secs() as usize
        } else {
            0
        };
        self.sink.append(source);
    }
    pub fn single_mode(&mut self) {
        if self.playlist.is_empty() {
            return;
        }
        let f = self.playlist[self.current_playlist_index].clone();
        if let Some(cur) = &self.current_audio {
            if f != *cur {
                self.stop();
                self.sink =
                    Sink::try_new(&self._stream_handle).expect("Can not init Sink and PlayError");
                self.current_audio = Some(f.clone());
                self.audio_event = AudioEvent::Play;
                self.append_source_to_sink_from_file(f);
            } else {
                if self.sink.len() < 1 {
                    self.append_source_to_sink_from_file(f);
                }
            }
        } else {
            self.current_audio = Some(f.clone());
            self.append_source_to_sink_from_file(f);
        }
    }
    pub fn playlist_mode(&mut self) -> Option<usize> {
        if self.playlist.is_empty() {
            return None;
        }
        if self.sink.len() < 1 {
            self.change_track_index();
            let f = self.playlist[self.current_playlist_index].clone();
            self.current_audio = Some(f.clone());
            self.append_source_to_sink_from_file(f);
            return Some(self.current_playlist_index);
        } else if self.sink.len() >= 1 {
            let f = self.playlist[self.current_playlist_index].clone();
            if let Some(cur) = &self.current_audio {
                if f != *cur {
                    self.stop();
                    self.sink = Sink::try_new(&self._stream_handle)
                        .expect("Can not init Sink and PlayError");
                    self.current_audio = Some(f.clone());
                    self.audio_event = AudioEvent::Play;
                    self.append_source_to_sink_from_file(f);
                }
            }
        }
        return None;
    }
    fn change_track_index(&mut self) {
        if self.loop_mode == LoopMode::Playlist {
            if self.current_playlist_index == self.playlist.len() - 1 {
                self.current_playlist_index = 0;
            } else {
                self.current_playlist_index += 1;
            }
        } else if self.loop_mode == LoopMode::Shuffle {
            loop {
                let new_idx = rand::rng().random_range(0..self.playlist.len());
                if new_idx != self.current_playlist_index {
                    self.current_playlist_index = new_idx;
                    break;
                }
            }
        }
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
    pub fn mute(&mut self) {
        self.sink.set_volume(0.0);
    }
    pub fn unmute(&mut self) {
        self.sink.set_volume(self.current_volume);
    }
    pub fn increase_vol(&mut self) {
        self.current_volume = (self.current_volume + 0.1).min(1.0);
        self.sink.set_volume(self.current_volume);
    }
    pub fn decrease_vol(&mut self) {
        self.current_volume = (self.current_volume - 0.1).max(0.0);
        self.sink.set_volume(self.current_volume);
    }
    pub fn get_current_position(&self) -> Duration {
        Duration::from_secs(self.sink.get_pos().as_secs() % (self.length as u64))
    }
}

struct WaveFormData {
    samples: Vec<f32>,
    sample_rate: usize,
    durations: usize,
}
impl WaveFormData {
    fn from_file(path: String) -> Self {
        let file = File::open(path).expect("Can not file this file");
        let source = Decoder::new(file).expect("Decoder Error");
        let sample_rate = source.sample_rate() as usize;
        let channels = source.channels() as usize;

        let samples: Vec<f32> = source
            .map(|s| (s as f32 / i16::MAX as f32).abs())
            .collect::<Vec<f32>>()
            .chunks(channels)
            .map(|ch| ch.iter().sum::<f32>() / channels as f32)
            .collect();
        let durations = samples.len() / sample_rate;
        Self {
            samples,
            sample_rate,
            durations,
        }
    }
}
