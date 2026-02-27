use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

/// Parse a byte stream into SSE events.
/// Handles `event:` and `data:` prefixes, multi-line data, and `[DONE]` sentinel.
pub fn parse_sse_stream(
    byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> Pin<Box<dyn Stream<Item = Result<SseEvent, ai_proxy_core::error::ProxyError>> + Send>> {
    let stream = async_stream(byte_stream);
    Box::pin(stream)
}

struct SseState {
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

fn async_stream(
    byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<SseEvent, ai_proxy_core::error::ProxyError>> + Send {
    futures::stream::unfold(
        SseState {
            stream: Box::pin(byte_stream),
            buffer: String::new(),
        },
        |mut state| async move {
            loop {
                // Check if we have a complete event block in the buffer (double newline)
                if let Some(pos) = find_event_boundary(&state.buffer) {
                    let block = state.buffer[..pos].to_string();
                    // Skip past the double newline
                    let skip = if state.buffer[pos..].starts_with("\r\n\r\n") {
                        4
                    } else {
                        2
                    };
                    state.buffer = state.buffer[pos + skip..].to_string();

                    if let Some(event) = parse_event_block(&block) {
                        return Some((Ok(event), state));
                    }
                    // Empty event block, continue looking
                    continue;
                }

                // Need more data
                match state.stream.next().await {
                    Some(Ok(bytes)) => match std::str::from_utf8(&bytes) {
                        Ok(text) => state.buffer.push_str(text),
                        Err(e) => {
                            return Some((
                                Err(ai_proxy_core::error::ProxyError::Internal(format!(
                                    "invalid UTF-8 in SSE stream: {e}"
                                ))),
                                state,
                            ));
                        }
                    },
                    Some(Err(e)) => {
                        return Some((
                            Err(ai_proxy_core::error::ProxyError::Network(e.to_string())),
                            state,
                        ));
                    }
                    None => {
                        // Stream ended. Process any remaining data in the buffer.
                        if !state.buffer.trim().is_empty() {
                            let block = std::mem::take(&mut state.buffer);
                            if let Some(event) = parse_event_block(&block) {
                                return Some((Ok(event), state));
                            }
                        }
                        return None;
                    }
                }
            }
        },
    )
}

/// Find the position of a double-newline event boundary.
fn find_event_boundary(s: &str) -> Option<usize> {
    if let Some(pos) = s.find("\n\n") {
        return Some(pos);
    }
    if let Some(pos) = s.find("\r\n\r\n") {
        return Some(pos);
    }
    None
}

/// Parse a single SSE event block into an SseEvent.
/// Returns None for empty/comment-only blocks and [DONE] sentinels.
fn parse_event_block(block: &str) -> Option<SseEvent> {
    let mut event_type: Option<String> = None;
    let mut data_lines: Vec<String> = Vec::new();

    for line in block.lines() {
        let line = line.trim_start_matches('\r');
        if line.starts_with(':') {
            // Comment line, skip
            continue;
        }
        if let Some(value) = line.strip_prefix("event:") {
            event_type = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("data:") {
            let value = value.trim_start();
            data_lines.push(value.to_string());
        } else if line.starts_with("id:") || line.starts_with("retry:") {
            // Ignore id and retry fields
        }
    }

    if data_lines.is_empty() {
        return None;
    }

    let data = data_lines.join("\n");

    Some(SseEvent {
        event: event_type,
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_event_block_basic() {
        let block = "data: {\"hello\": \"world\"}";
        let event = parse_event_block(block).unwrap();
        assert!(event.event.is_none());
        assert_eq!(event.data, "{\"hello\": \"world\"}");
    }

    #[test]
    fn test_parse_event_block_with_event_type() {
        let block = "event: message_start\ndata: {\"type\": \"message_start\"}";
        let event = parse_event_block(block).unwrap();
        assert_eq!(event.event.as_deref(), Some("message_start"));
        assert_eq!(event.data, "{\"type\": \"message_start\"}");
    }

    #[test]
    fn test_parse_event_block_done() {
        let block = "data: [DONE]";
        let event = parse_event_block(block).unwrap();
        assert_eq!(event.data, "[DONE]");
    }

    #[test]
    fn test_parse_event_block_multiline_data() {
        let block = "data: line1\ndata: line2";
        let event = parse_event_block(block).unwrap();
        assert_eq!(event.data, "line1\nline2");
    }

    #[test]
    fn test_parse_event_block_comment() {
        let block = ": this is a comment";
        assert!(parse_event_block(block).is_none());
    }
}
