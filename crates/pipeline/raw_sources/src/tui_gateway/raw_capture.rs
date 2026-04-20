use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::mpsc;

use crate::acp_client::{ChatSessionId, RawEventFanout, RawEventPublisher};

const READ_CHUNK_SIZE: usize = 8192;

pub async fn capture_gateway_stream<R>(
    session_id: ChatSessionId,
    sequence: Arc<AtomicU64>,
    mut reader: R,
    publisher: Arc<dyn RawEventPublisher>,
    fanout: RawEventFanout,
    frames_tx: Option<mpsc::UnboundedSender<Bytes>>,
) where
    R: AsyncRead + Unpin,
{
    let mut buffer = BytesMut::with_capacity(READ_CHUNK_SIZE);
    loop {
        match reader.read_buf(&mut buffer).await {
            Ok(0) => break,
            Ok(_) => {
                publish_complete_lines(
                    &mut buffer,
                    &session_id,
                    &sequence,
                    &publisher,
                    &fanout,
                    frames_tx.as_ref(),
                )
                .await
            }
            Err(_) => break,
        }
    }
    publish_remainder(
        &mut buffer,
        &session_id,
        &sequence,
        &publisher,
        &fanout,
        frames_tx.as_ref(),
    )
    .await;
}

async fn publish_complete_lines(
    buffer: &mut BytesMut,
    session_id: &ChatSessionId,
    sequence: &AtomicU64,
    publisher: &Arc<dyn RawEventPublisher>,
    fanout: &RawEventFanout,
    frames_tx: Option<&mpsc::UnboundedSender<Bytes>>,
) {
    while let Some(newline_index) = buffer.iter().position(|byte| *byte == b'\n') {
        let frame = buffer.split_to(newline_index + 1).freeze();
        publish_frame(session_id, sequence, publisher, fanout, frames_tx, frame).await;
    }
}

async fn publish_remainder(
    buffer: &mut BytesMut,
    session_id: &ChatSessionId,
    sequence: &AtomicU64,
    publisher: &Arc<dyn RawEventPublisher>,
    fanout: &RawEventFanout,
    frames_tx: Option<&mpsc::UnboundedSender<Bytes>>,
) {
    if buffer.is_empty() {
        return;
    }
    let frame = buffer.split().freeze();
    publish_frame(session_id, sequence, publisher, fanout, frames_tx, frame).await;
}

async fn publish_frame(
    session_id: &ChatSessionId,
    sequence: &AtomicU64,
    publisher: &Arc<dyn RawEventPublisher>,
    fanout: &RawEventFanout,
    frames_tx: Option<&mpsc::UnboundedSender<Bytes>>,
    frame: Bytes,
) {
    let seq = sequence.fetch_add(1, Ordering::Relaxed);
    let _ = publisher
        .publish_raw_event(session_id, seq, frame.clone())
        .await;
    fanout.publish(session_id.clone(), seq, frame.clone());
    if let Some(tx) = frames_tx {
        let _ = tx.send(frame);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::acp_client::RawEventPublishError;

    #[derive(Default)]
    struct RecordingPublisher {
        frames: Mutex<Vec<Bytes>>,
    }

    #[async_trait]
    impl RawEventPublisher for RecordingPublisher {
        async fn publish_raw_event(
            &self,
            _session_id: &ChatSessionId,
            _sequence: u64,
            raw_bytes: Bytes,
        ) -> Result<(), RawEventPublishError> {
            self.frames.lock().unwrap().push(raw_bytes);
            Ok(())
        }
    }

    #[tokio::test]
    async fn captures_unknown_gateway_events_without_interpreting_them() {
        let publisher = Arc::new(RecordingPublisher::default());
        let (mut writer, reader) = tokio::io::duplex(1024);
        let (frames_tx, mut frames_rx) = mpsc::unbounded_channel();
        let session_id = ChatSessionId::from("tui-gateway-test");

        let task = tokio::spawn(capture_gateway_stream(
            session_id,
            Arc::new(AtomicU64::new(1)),
            reader,
            publisher.clone(),
            RawEventFanout::default(),
            Some(frames_tx),
        ));

        let unknown = br#"{"jsonrpc":"2.0","method":"event","params":{"type":"new.event.tomorrow","payload":{"x":1}}}"#;
        writer.write_all(unknown).await.unwrap();
        writer.write_all(b"\n").await.unwrap();
        drop(writer);
        task.await.unwrap();

        let captured = frames_rx.recv().await.unwrap();
        assert_eq!(captured.as_ref(), [unknown.as_slice(), b"\n"].concat());
        assert_eq!(
            publisher.frames.lock().unwrap()[0].as_ref(),
            [unknown.as_slice(), b"\n"].concat()
        );
    }
}
