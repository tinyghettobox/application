use std::io::{Read, Seek, SeekFrom};
use std::num::NonZeroUsize;
use stream_download::http::reqwest::{Client, Url};

use stream_download::http::HttpStream;
use stream_download::source::SourceStream;
use stream_download::storage::bounded::BoundedStorageProvider;
use stream_download::storage::memory::MemoryStorageProvider;
use stream_download::{Settings, StreamDownload};
use symphonia::core::io::MediaSource;

pub struct RemoteMediaSource {
    reader: StreamDownload<MemoryStorageProvider>,
    content_length: Option<u64>,
}

impl RemoteMediaSource {
    pub async fn from_url(url: String) -> Result<Self, String> {
        let parsed_url = url.parse::<Url>().map_err(|error| format!("Could not parse url: {}", error))?;
        let stream = HttpStream::<Client>::create(parsed_url)
            .await
            .map_err(|error| format!("Could not create stream: {}", error))?;

        let content_length = stream.content_length();
        // let bitrate: u64 = stream.header("Icy-Br").unwrap_or("320").parse().unwrap_or(320);

        let reader = StreamDownload::from_stream(
            stream,
            // BoundedStorageProvider::new(MemoryStorageProvider, NonZeroUsize::new(512 * 1024).unwrap()),
            MemoryStorageProvider::default(),
            Settings::default().prefetch_bytes(320 / 8 * 1024 * 5),
        )
        .await
        .map_err(|error| format!("Could start download: {}", error))?;

        Ok(RemoteMediaSource { reader, content_length })
    }
}

impl Read for RemoteMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.reader.read(buf)?;

        Ok(len)
    }
}

impl Seek for RemoteMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}

impl MediaSource for RemoteMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.content_length
    }
}
