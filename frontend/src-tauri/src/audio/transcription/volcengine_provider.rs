use async_trait::async_trait;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use futures_util::{SinkExt, StreamExt};
use log::{debug, warn};
use serde_json::{json, Value};
use std::io::{Read, Write};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::provider::{TranscriptResult, TranscriptionError, TranscriptionProvider};

const DEFAULT_ENDPOINT: &str = "wss://openspeech.bytedance.com/api/v3/sauc/bigmodel_nostream";

pub struct VolcengineProvider {
    app_key: String,
    access_key: String,
    resource_id: String,
    endpoint: String,
    model: String,
}

impl VolcengineProvider {
    pub fn new(
        app_key: String,
        access_key: String,
        resource_id: String,
        endpoint: Option<String>,
        model: String,
    ) -> Result<Self, TranscriptionError> {
        if app_key.trim().is_empty() || access_key.trim().is_empty() {
            return Err(TranscriptionError::EngineFailed(
                "Volcengine ASR requires an App Key and Access Token".to_string(),
            ));
        }
        if resource_id.trim().is_empty() {
            return Err(TranscriptionError::EngineFailed(
                "Volcengine ASR requires a Resource ID".to_string(),
            ));
        }
        Ok(Self {
            app_key,
            access_key,
            resource_id,
            endpoint: endpoint
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string()),
            model,
        })
    }

    fn gzip(data: &[u8]) -> Result<Vec<u8>, TranscriptionError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;
        encoder
            .finish()
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))
    }

    fn gunzip(data: &[u8]) -> Result<Vec<u8>, TranscriptionError> {
        let mut decoder = GzDecoder::new(data);
        let mut output = Vec::new();
        decoder
            .read_to_end(&mut output)
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;
        Ok(output)
    }

    fn gzip_frame(header: [u8; 4], payload: &[u8]) -> Result<Vec<u8>, TranscriptionError> {
        let compressed = Self::gzip(payload)?;
        let mut frame = Vec::with_capacity(8 + compressed.len());
        frame.extend_from_slice(&header);
        frame.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
        frame.extend_from_slice(&compressed);
        Ok(frame)
    }

    fn pcm16(audio: &[f32]) -> Vec<u8> {
        let mut pcm = Vec::with_capacity(audio.len() * 2);
        for sample in audio {
            let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
            pcm.extend_from_slice(&value.to_le_bytes());
        }
        pcm
    }

    fn parse_response(data: &[u8]) -> Result<Option<String>, TranscriptionError> {
        if data.len() < 8 {
            return Err(TranscriptionError::EngineFailed(
                "Volcengine ASR returned an invalid response frame".to_string(),
            ));
        }
        let message_type = data[1] >> 4;
        if message_type == 0x0f {
            return Err(TranscriptionError::EngineFailed(
                String::from_utf8_lossy(&data[8..]).to_string(),
            ));
        }
        let flags = data[1] & 0x0f;
        let compression = data[2] & 0x0f;
        let mut offset = 4;
        if flags == 0x01 || flags == 0x03 {
            offset += 4;
        }
        if data.len() < offset + 4 {
            return Ok(None);
        }
        let size = u32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        if data.len() < offset + size {
            return Err(TranscriptionError::EngineFailed(
                "Volcengine ASR returned a truncated response frame".to_string(),
            ));
        }
        let payload = &data[offset..offset + size];
        let payload = if compression == 0x01 {
            Self::gunzip(payload)?
        } else {
            payload.to_vec()
        };
        let value: Value = serde_json::from_slice(&payload)
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;
        Ok(value
            .get("result")
            .and_then(|result| result.get("text"))
            .and_then(Value::as_str)
            .map(str::to_string))
    }
}

#[async_trait]
impl TranscriptionProvider for VolcengineProvider {
    async fn transcribe(
        &self,
        audio: Vec<f32>,
        language: Option<String>,
    ) -> Result<TranscriptResult, TranscriptionError> {
        if audio.is_empty() {
            return Err(TranscriptionError::AudioTooShort {
                samples: 0,
                minimum: 1,
            });
        }

        let request = json!({
            "user": { "uid": "meetily-local" },
            "audio": {
                "format": "pcm",
                "codec": "raw",
                "rate": 16000,
                "bits": 16,
                "channel": 1,
                "language": language.as_deref().and_then(|value| match value {
                    "zh" | "zh-CN" => Some("zh-CN"),
                    "en" | "en-US" => Some("en-US"),
                    _ => None,
                }),
            },
            "request": {
                "model_name": if self.model.trim().is_empty() { "bigmodel" } else { &self.model },
                "enable_itn": true,
                "enable_punc": true,
                "show_utterances": false,
            }
        });

        // The v3 protocol uses JSON + gzip for the initial request and gzip for
        // the terminal audio frame. The last header byte is reserved and must
        // remain zero; the low nibble of byte 2 advertises gzip compression.
        let full_frame =
            Self::gzip_frame([0x11, 0x10, 0x11, 0x00], request.to_string().as_bytes())?;
        let pcm = Self::pcm16(&audio);
        let builder = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(&self.endpoint)
            .header("X-Api-Key", &self.app_key)
            .header("X-Api-App-Key", &self.app_key)
            .header("X-Api-Access-Key", &self.access_key)
            .header("X-Api-Resource-Id", &self.resource_id)
            .header("X-Api-Request-Id", uuid::Uuid::new_v4().to_string())
            .header("X-Api-Connect-Id", uuid::Uuid::new_v4().to_string());
        let request = builder
            .body(())
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;
        let (mut socket, response) = connect_async(request)
            .await
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;
        debug!(
            "Volcengine ASR websocket connected: {:?}",
            response.status()
        );
        socket
            .send(Message::Binary(full_frame.into()))
            .await
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;

        // 0x2 is an audio-only client message and flag 0x2 marks the final
        // frame, so the no-stream endpoint can emit the result immediately.
        let audio_frame = Self::gzip_frame([0x11, 0x22, 0x01, 0x00], &pcm)?;
        socket
            .send(Message::Binary(audio_frame.into()))
            .await
            .map_err(|error| TranscriptionError::EngineFailed(error.to_string()))?;

        let mut text = None;
        while let Some(message) = socket.next().await {
            match message.map_err(|error| TranscriptionError::EngineFailed(error.to_string()))? {
                Message::Binary(data) => {
                    if let Some(result) = Self::parse_response(&data)? {
                        text = Some(result);
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
            if text.is_some() {
                break;
            }
        }

        if text.is_none() {
            warn!("Volcengine ASR returned no text");
        }
        Ok(TranscriptResult {
            text: text.unwrap_or_default(),
            confidence: None,
            is_partial: false,
        })
    }

    async fn is_model_loaded(&self) -> bool {
        true
    }

    async fn get_current_model(&self) -> Option<String> {
        Some(self.model.clone())
    }

    fn provider_name(&self) -> &'static str {
        "Volcengine ASR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm16_is_little_endian_and_clamped() {
        assert_eq!(
            VolcengineProvider::pcm16(&[-2.0, -1.0, 0.0, 1.0, 2.0]),
            vec![0x01, 0x80, 0x01, 0x80, 0x00, 0x00, 0xff, 0x7f, 0xff, 0x7f,]
        );
    }

    #[test]
    fn gzip_frame_advertises_gzip_and_round_trips_payload() {
        let payload = r#"{"result":{"text":"你好"}}"#;
        let frame = VolcengineProvider::gzip_frame([0x11, 0x10, 0x11, 0x00], payload.as_bytes())
            .expect("frame should be encoded");
        assert_eq!(&frame[..4], &[0x11, 0x10, 0x11, 0x00]);
        let size = u32::from_be_bytes(frame[4..8].try_into().unwrap()) as usize;
        assert_eq!(size, frame.len() - 8);
        assert_eq!(
            VolcengineProvider::gunzip(&frame[8..]).unwrap(),
            payload.as_bytes()
        );
    }

    #[test]
    fn parse_response_reads_compressed_json_text() {
        let payload = r#"{"result":{"text":"你好，会议开始"}}"#;
        let compressed = VolcengineProvider::gzip(payload.as_bytes()).unwrap();
        let mut frame = vec![0x11, 0x09, 0x01, 0x00];
        frame.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
        frame.extend_from_slice(&compressed);
        assert_eq!(
            VolcengineProvider::parse_response(&frame).unwrap(),
            Some("你好，会议开始".to_string())
        );
    }
}
