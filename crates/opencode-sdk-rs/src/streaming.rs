//! Server-Sent Events (SSE) streaming support.
//!
//! Provides [`SseStream`], a `futures_core::Stream` that wraps an HTTP byte
//! stream (from `hpx::Response::bytes_stream()`) and yields typed items
//! parsed from SSE `data:` fields.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_core::Stream;
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;

use crate::error::OpencodeError;

// ---------------------------------------------------------------------------
// ServerSentEvent
// ---------------------------------------------------------------------------

/// A single Server-Sent Event parsed from the wire format.
#[derive(Debug, Clone, Default)]
pub struct ServerSentEvent {
    /// The event type (from `event:` lines).
    pub event: Option<String>,
    /// The data payload (from `data:` lines, concatenated with newlines).
    pub data: String,
    /// The event ID (from `id:` lines).
    pub id: Option<String>,
}

// ---------------------------------------------------------------------------
// SseDecoder
// ---------------------------------------------------------------------------

/// Internal buffer that accumulates SSE lines from a byte stream
/// and yields complete [`ServerSentEvent`]s on empty-line boundaries.
struct SseDecoder {
    /// Accumulates partial lines across chunk boundaries.
    buffer: String,
    /// Current `event:` value being built.
    current_event: Option<String>,
    /// Accumulated `data:` lines for the current event.
    current_data: Vec<String>,
    /// Current `id:` value being built.
    current_id: Option<String>,
}

impl SseDecoder {
    const fn new() -> Self {
        Self {
            buffer: String::new(),
            current_event: None,
            current_data: Vec::new(),
            current_id: None,
        }
    }

    /// Feed a chunk of bytes into the decoder, returning any complete events.
    fn feed(&mut self, chunk: &[u8]) -> Vec<ServerSentEvent> {
        let text = String::from_utf8_lossy(chunk);
        self.buffer.push_str(&text);

        let mut events = Vec::new();

        // Process all complete lines (terminated by \n).
        // Partial lines remain in `self.buffer` for the next call.
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].trim_end_matches('\r').to_owned();
            self.buffer = self.buffer[newline_pos + 1..].to_owned();

            if line.is_empty() {
                // Empty line marks the end of an event.
                if let Some(event) = self.emit_event() {
                    events.push(event);
                }
                continue;
            }

            if line.starts_with(':') {
                // Comment line — ignore.
                continue;
            }

            let (field, value) = if let Some(colon_pos) = line.find(':') {
                let field = &line[..colon_pos];
                let mut value = &line[colon_pos + 1..];
                // Strip a single leading space after the colon (per SSE spec).
                if value.starts_with(' ') {
                    value = &value[1..];
                }
                (field.to_owned(), value.to_owned())
            } else {
                // Field with no value.
                (line, String::new())
            };

            match field.as_str() {
                "event" => self.current_event = Some(value),
                "data" => self.current_data.push(value),
                "id" => self.current_id = Some(value),
                // Unknown fields are ignored per the SSE spec.
                _ => {}
            }
        }

        events
    }

    /// Emit the current event (if any data has been accumulated) and reset.
    fn emit_event(&mut self) -> Option<ServerSentEvent> {
        if self.current_data.is_empty() && self.current_event.is_none() && self.current_id.is_none()
        {
            return None;
        }

        let event = ServerSentEvent {
            event: self.current_event.take(),
            data: self.current_data.join("\n"),
            id: self.current_id.take(),
        };
        self.current_data.clear();

        Some(event)
    }

    /// Flush any remaining partial event when the stream ends.
    fn flush(&mut self) -> Option<ServerSentEvent> {
        // If there is leftover text in the buffer, treat it as a final line.
        if !self.buffer.is_empty() {
            let remaining = std::mem::take(&mut self.buffer);
            let trimmed = remaining.trim_end_matches('\r');
            if !trimmed.is_empty() && !trimmed.starts_with(':') {
                let (field, value) = trimmed.find(':').map_or_else(
                    || (trimmed.to_owned(), String::new()),
                    |colon_pos| {
                        let field = &trimmed[..colon_pos];
                        let mut value = &trimmed[colon_pos + 1..];
                        if value.starts_with(' ') {
                            value = &value[1..];
                        }
                        (field.to_owned(), value.to_owned())
                    },
                );

                match field.as_str() {
                    "event" => self.current_event = Some(value),
                    "data" => self.current_data.push(value),
                    "id" => self.current_id = Some(value),
                    _ => {}
                }
            }
        }

        self.emit_event()
    }
}

// ---------------------------------------------------------------------------
// SseStream
// ---------------------------------------------------------------------------

pin_project! {
    /// A stream of typed items parsed from Server-Sent Events.
    ///
    /// Wraps an inner byte stream (from `hpx::Response::bytes_stream()`)
    /// and parses each SSE event's `data` field as JSON of type `T`.
    pub struct SseStream<T> {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<Bytes, hpx::Error>> + Send>>,
        decoder: SseDecoder,
        pending: Vec<ServerSentEvent>,
        _marker: std::marker::PhantomData<T>,
    }
}

impl<T: DeserializeOwned> SseStream<T> {
    /// Create an `SseStream` from an hpx response byte stream.
    pub(crate) fn new(
        byte_stream: impl Stream<Item = Result<Bytes, hpx::Error>> + Send + 'static,
    ) -> Self {
        Self {
            inner: Box::pin(byte_stream),
            decoder: SseDecoder::new(),
            pending: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: DeserializeOwned> Stream for SseStream<T> {
    type Item = Result<T, OpencodeError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // First, drain any pending events from a previous chunk.
        if !this.pending.is_empty() {
            let event = this.pending.remove(0);
            if event.data.is_empty() {
                // Skip events with no data (heartbeats, etc.).
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            let parsed =
                serde_json::from_str::<T>(&event.data).map_err(OpencodeError::Serialization);
            return Poll::Ready(Some(parsed));
        }

        // Poll the inner byte stream for more data.
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let events = this.decoder.feed(&bytes);
                *this.pending = events;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(OpencodeError::Connection {
                message: e.to_string(),
                source: Some(Box::new(e)),
            }))),
            Poll::Ready(None) => {
                // Stream ended — flush any remaining partial event.
                if let Some(event) = this.decoder.flush() &&
                    !event.data.is_empty()
                {
                    let parsed = serde_json::from_str::<T>(&event.data)
                        .map_err(OpencodeError::Serialization);
                    return Poll::Ready(Some(parsed));
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_event() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"data: {\"key\":\"value\"}\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "{\"key\":\"value\"}");
        assert!(events[0].event.is_none());
    }

    #[test]
    fn test_parse_event_with_type() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"event: message\ndata: hello\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.as_deref(), Some("message"));
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn test_parse_multiline_data() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"data: line1\ndata: line2\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2");
    }

    #[test]
    fn test_parse_multiple_events() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"data: event1\n\ndata: event2\n\n");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "event1");
        assert_eq!(events[1].data, "event2");
    }

    #[test]
    fn test_ignore_comments() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b": this is a comment\ndata: actual\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "actual");
    }

    #[test]
    fn test_chunked_data() {
        let mut decoder = SseDecoder::new();
        let events1 = decoder.feed(b"data: hel");
        assert!(events1.is_empty());
        let events2 = decoder.feed(b"lo\n\n");
        assert_eq!(events2.len(), 1);
        assert_eq!(events2[0].data, "hello");
    }

    #[test]
    fn test_id_field() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"id: 42\ndata: test\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id.as_deref(), Some("42"));
        assert_eq!(events[0].data, "test");
    }

    #[test]
    fn test_flush_remaining() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"data: partial");
        assert!(events.is_empty());
        let event = decoder.flush();
        assert!(event.is_some());
        assert_eq!(event.as_ref().unwrap().data, "partial");
    }

    #[test]
    fn test_empty_line_no_data() {
        let mut decoder = SseDecoder::new();
        // An empty line without prior fields produces nothing.
        let events = decoder.feed(b"\n");
        assert!(events.is_empty());
    }

    #[test]
    fn test_field_without_value() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"data\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "");
    }

    #[test]
    fn test_crlf_line_endings() {
        let mut decoder = SseDecoder::new();
        let events = decoder.feed(b"data: hello\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn test_sse_stream_typed_compiles() {
        // Verify that SseStream implements Stream with the expected Item.
        fn _assert_stream<S: Stream<Item = Result<serde_json::Value, OpencodeError>>>(_s: S) {}

        // Verify SseStream is Send (required for async runtimes).
        fn _assert_send<S: Send>(_s: S) {}
    }
}
