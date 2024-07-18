use reqwest::{Client, Url};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use stream_download::http::HttpStream;
use stream_download::source::SourceStream;
use stream_download::storage::memory::MemoryStorageProvider;
use stream_download::{Settings, StreamDownload};
use symphonia::core::io::MediaSource;

struct TrackInfo {
    duration: Duration,
    sample_rate: i64,
    position: i64,
    offset: i64,
}

impl TrackInfo {
    fn set_position(&mut self, position: Duration) {
        self.offset = position.as_secs() as i64 * self.sample_rate - self.position;
    }

    fn seek_forward(&mut self, duration: Duration) {
        self.offset = duration.as_secs() as i64 * self.sample_rate;
    }

    fn seek_backward(&mut self, duration: Duration) {
        self.offset = duration.as_secs() as i64 * self.sample_rate * -1;
    }
}

struct SeekableDecoder {
    decoder: Decoder<BufReader<File>>,
    info: Arc<Mutex<TrackInfo>>,
}

impl SeekableDecoder {
    fn new(file_path: &str) -> Self {
        let file = File::open(file_path).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        let duration = decoder.total_duration().unwrap_or(Duration::from_secs(0));
        let sample_rate = decoder.sample_rate() as i64;

        SeekableDecoder {
            decoder,
            info: Arc::new(Mutex::new(TrackInfo {
                duration,
                sample_rate,
                position: 0,
                offset: 0,
            })),
        }
    }

    fn control(&self) -> Arc<Mutex<TrackInfo>> {
        self.info.clone()
    }
}

impl Source for SeekableDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        self.decoder.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.decoder.total_duration()
    }
}

impl Iterator for SeekableDecoder {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let mut info = self.info.lock().unwrap();
        if info.offset != 0 {
            println!("Setting offset {}", info.offset as usize);
            info.position += info.offset;
            let mut skipped = self.decoder.by_ref().skip(info.offset as usize);
            info.offset = 0;
            return skipped.next();
        }

        info.position += 1;
        self.decoder.next()
    }
}

struct RemoteSource {
    reader: StreamDownload<MemoryStorageProvider>,
    content_length: Option<u64>,
}

impl RemoteSource {
    pub async fn from_url(url: String) -> Result<Self, String> {
        let parsed_url = url.parse::<Url>().map_err(|error| format!("Invalid url: {}", error))?;
        let stream = HttpStream::<Client>::create(parsed_url)
            .await
            .map_err(|error| format!("Failed to create stream: {}", error))?;

        let content_length = stream.content_length();
        let reader = StreamDownload::from_stream(stream, MemoryStorageProvider::default(), Settings::default())
            .await
            .map_err(|error| format!("Failed to create download: {}", error))?;

        Ok(RemoteSource { reader, content_length })
    }
}

impl Read for RemoteSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Seek for RemoteSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl MediaSource for RemoteSource {
    fn is_seekable(&self) -> bool {
        self.content_length.is_some()
    }

    fn byte_len(&self) -> Option<u64> {
        self.content_length
    }
}

#[tokio::main]
async fn main() {
    // Decode a song from a file
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // let mut seekable_decoder = SeekableDecoder::new(FILE);

    // Seek forward 10 seconds
    // let control = seekable_decoder.control();

    // sink.append(seekable_decoder);
    let url = "https://streamtdy.ir-media-tec.com/live/mp3-128/web/play.mp3".to_string();

    let file = RemoteSource::from_url(url).await.expect("asd");
    let decode = Decoder::new(file).expect("decoder");
    // sink.play();
    sink.append(decode);
    sink.play();

    println!("Playing");
    sleep(Duration::from_secs(5)); // Sleep for 60 seconds to allow playback

    // println!("Seeking forward");
    // sink.pause();
    // control.lock().unwrap().seek_forward(Duration::from_secs(20));
    // sink.play();

    // println!("waiting");
    // sleep(Duration::from_secs(5)); // Sleep for 60 seconds to allow playback

    // println!("Seeking backward");
    // sink.pause();
    // control.lock().unwrap().seek_backward(Duration::from_secs(5));
    // sink.play();

    // println!("waiting");
    // sleep(Duration::from_secs(20)); // Sleep for 60 seconds to allow playback

    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let sink = Sink::try_new(&stream_handle).unwrap();
    //
    // println!("start");
    // let file = File::open("H:\\Data\\projects\\mupibox-rs\\admin_interface\\.storage\\audio\\0b7678f1-4121-4d38-be75-c26f86e2e30d-IsItReal.mp3").expect("could not read file");
    // let source = Decoder::new(BufReader::new(file)).expect("Could not decode file");
    //
    // println!("appending");
    // sink.append(source);
    // sink.set_volume(0.3);
    // sink.play();
    //
    // println!("Sleep");
    // sleep(Duration::from_secs(1));
    //
    // println!("repeat");
    //
    // let file2 = File::open("H:\\Data\\projects\\mupibox-rs\\admin_interface\\.storage\\audio\\0b7678f1-4121-4d38-be75-c26f86e2e30d-IsItReal.mp3").expect("could not read file");
    // let mut source2 = Decoder::new(BufReader::new(file2)).expect("Could not decode file");
    //
    // println!("skipping");
    // let skipped = source2.skip_duration(Duration::from_secs(130));
    // println!("clearing");
    // sink.clear();
    // println!("appending");
    // sink.append(skipped);
    // println!("plazing");
    // sink.play();
    // println!("play");
    //
    // sink.sleep_until_end();
}
