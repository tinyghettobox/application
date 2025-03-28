pub mod decoder;
pub mod remote_decoder;
pub mod media_source;

/// Helper crate to create a remote stream with kira
///
/// ```
/// use kira_remote_stream::RemoteStreamDecoder;
/// use kira::sound::streaming::{StreamingSoundData, StreamingSoundSettings};
///
/// let settings = StreamingSoundSettings::default();
/// let stream = StreamingSoundData::from_decoder(RemoteStreamDecoder::from_url("http://".to_string())?, settings);
/// ```
pub use decoder::symphonia::SymphoniaDecoder;
pub use remote_decoder::RemoteStreamDecoder;
