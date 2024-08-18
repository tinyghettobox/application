use std::env;
use std::thread::sleep;
use std::time::Duration;

use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundSettings};
use kira::tween::Value;
use kira::Volume;

// struct RemoteSource {
//     reader: StreamDownload<MemoryStorageProvider>,
//     content_length: Option<u64>,
// }
//
// impl RemoteSource {
//     pub async fn from_url(url: String) -> Result<Self, String> {
//         let parsed_url = url.parse::<Url>().map_err(|error| format!("Invalid url: {}", error))?;
//         let stream = HttpStream::<Client>::create(parsed_url)
//             .await
//             .map_err(|error| format!("Failed to create stream: {}", error))?;
//
//         let content_length = stream.content_length();
//         let reader = StreamDownload::from_stream(stream, MemoryStorageProvider::default(), Settings::default())
//             .await
//             .map_err(|error| format!("Failed to create download: {}", error))?;
//
//         Ok(RemoteSource { reader, content_length })
//     }
// }
//
// impl Read for RemoteSource {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         self.reader.read(buf)
//     }
// }
//
// impl Seek for RemoteSource {
//     fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
//         self.reader.seek(pos)
//     }
// }
//
// impl MediaSource for RemoteSource {
//     fn is_seekable(&self) -> bool {
//         self.content_length.is_some()
//     }
//
//     fn byte_len(&self) -> Option<u64> {
//         self.content_length
//     }
// }

const FILE: &'static str = "H:\\Data\\projects\\mupibox-rs\\admin_interface\\.storage\\audio\\0b7678f1-4121-4d38-be75-c26f86e2e30d-IsItReal.mp3";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Kira {:?}", args);
    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).expect("manager to load");

    // let url = "https://streamtdy.ir-media-tec.com/live/mp3-128/web/play.mp3".to_string();

    // let mut player = player::Player::default();
    // player.play_stream(url).await.expect("to play");
    //
    let sound = StreamingSoundData::from_file(
        args.iter().nth(1).expect("Url to be passed"),
        StreamingSoundSettings::new().volume(Value::Fixed(Volume::Amplitude(0.3))),
    )
        .expect("stream sound data to be created");
    //
    let mut handle = manager.play(sound).expect("failed to play");

    sleep(Duration::from_secs(50));
}
