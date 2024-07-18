use crate::decoder::symphonia::SymphoniaDecoder;
use crate::media_source::RemoteMediaSource;

pub struct RemoteStreamDecoder {}

impl RemoteStreamDecoder {
    pub async fn from_url(url: String) -> Result<SymphoniaDecoder, String> {
        let stream = RemoteMediaSource::from_url(url)
            .await
            .map_err(|error| format!("Could not create remote media source: {}", error))?;
        let decoder = SymphoniaDecoder::new(Box::new(stream))
            .map_err(|error| format!("Could not create remote decoder: {}", error))?;

        Ok(decoder)
    }
}
