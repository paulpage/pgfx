use std::fs::File;
use std::sync::Arc;
use std::io::{BufReader, Cursor};
use std::path::Path;
use rodio::{Decoder, decoder::LoopedDecoder, OutputStream, OutputStreamHandle, Sink};
use rodio::source::{Buffered, Source};

pub struct Sound {
    data: Arc<[u8]>,
}

impl Sound {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: Arc::from(bytes),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Self {
        Self {
            data: Arc::from(std::fs::read(path).unwrap()),
        }
    }
}

pub struct SoundEngine {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sinks: Vec<Sink>,
    next_sink: usize,
}

impl SoundEngine {
    pub fn new() -> Self {
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        let mut sinks = Vec::new();
        for _ in 0..8 {
            sinks.push(Sink::try_new(&_stream_handle).unwrap());
        }

        Self {
            _stream,
            _stream_handle,
            sinks,
            next_sink: 1,
        }
    }

    pub fn play(&mut self, sound: &Sound) {
        // TODO detect free sinks
        let sink_idx = self.next_sink;
        self.next_sink += 1;
        if self.next_sink == 8 {
            self.next_sink = 1;
        }

        let source = Decoder::new(Cursor::new(Arc::clone(&sound.data))).unwrap();
        self.sinks[sink_idx].append(source);
        self.sinks[sink_idx].play();
    }

    pub fn play_music(&mut self, sound: &Sound) {
        self.sinks[0].clear();

        let source = Decoder::new(Cursor::new(Arc::clone(&sound.data))).unwrap();
        self.sinks[0].append(source.repeat_infinite());
        self.sinks[0].play();
    }

    pub fn pause_music(&mut self) {
        self.sinks[0].pause();
    }

    pub fn resume_music(&mut self) {
        self.sinks[0].play();
    }
}
