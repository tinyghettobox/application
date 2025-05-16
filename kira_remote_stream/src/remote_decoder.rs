use crate::decoder::symphonia::SymphoniaDecoder;
use crate::media_source::RemoteMediaSource;

pub struct RemoteStreamDecoder {}

impl RemoteStreamDecoder {
    pub async fn from_url(url: String) -> Result<SymphoniaDecoder, String> {
        let stream = Self::try_create_stream(url.clone()).await?;
        let decoder = tokio::task::spawn_blocking(move || {
            SymphoniaDecoder::new(Box::new(stream))
                .map_err(|error| format!("Could not create remote decoder: {}", error))
        })
        .await
        .unwrap()?;

        Ok(decoder)
    }

    async fn try_create_stream(url: String) -> Result<RemoteMediaSource, String> {
        for attempt in 0..3 {
            match RemoteMediaSource::from_url(url.clone()).await {
                Ok(stream) => return Ok(stream),
                Err(error) => {
                    if attempt == 2 {
                        return Err(format!(
                            "Failed to create stream after 3 attempts: {}",
                            error
                        ));
                    }
                }
            }
        }
        Err("Failed to create stream, some coding error".to_string())
    }
}
