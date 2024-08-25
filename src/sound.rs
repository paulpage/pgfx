use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, decoder::LoopedDecoder, OutputStream, OutputStreamHandle, Sink};
use rodio::source::{Buffered, Source};

pub type Sound = Buffered<LoopedDecoder<BufReader<File>>>;

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

    pub fn load(&mut self, path: &str) -> Sound {
        let f = BufReader::new(File::open(path).unwrap());
        Decoder::new_looped(f).unwrap().buffered()
    }

    pub fn play(&mut self, sound: &Sound) {
        // TODO detect free sinks
        let sink_idx = self.next_sink;
        self.next_sink += 1;
        if self.next_sink == 8 {
            self.next_sink = 1;
        }
        self.sinks[sink_idx].append(sound.clone());
        self.sinks[sink_idx].play();
    }

    pub fn play_music(&mut self, sound: &Sound) {
        self.sinks[0].clear();
        self.sinks[0].append(sound.clone().repeat_infinite());
        self.sinks[0].play();
    }

    pub fn pause_music(&mut self) {
        self.sinks[0].pause();
    }

    pub fn resume_music(&mut self) {
        self.sinks[0].play();
    }
}
