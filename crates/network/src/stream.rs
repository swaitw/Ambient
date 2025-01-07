use std::{any::type_name, marker::PhantomData, pin::Pin, task::Poll};

use bytes::{Bytes, BytesMut};
use futures::{ready, Sink, SinkExt, Stream};
use pin_project::pin_project;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{NetworkError, MAX_FRAME_SIZE};

/// Transport agnostic framed reader
#[pin_project]
pub struct RawFramedRecvStream<S>(#[pin] FramedRead<S, LengthDelimitedCodec>);

impl<S> RawFramedRecvStream<S>
where
    S: AsyncRead,
{
    pub fn new(stream: S) -> Self {
        let mut codec = LengthDelimitedCodec::new();
        codec.set_max_frame_length(MAX_FRAME_SIZE);
        Self(FramedRead::new(stream, codec))
    }
}

impl<S> Stream for RawFramedRecvStream<S>
where
    S: AsyncRead,
{
    type Item = Result<Bytes, NetworkError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let p = self.project();

        let ready_result = ready!(p.0.poll_next(cx));
        Poll::Ready(ready_result.map(|result| result.map(BytesMut::freeze).map_err(Into::into)))
    }
}

/// Typed transport agnostic framed reader
#[pin_project]
pub struct FramedRecvStream<T, S> {
    #[pin]
    read: FramedRead<S, LengthDelimitedCodec>,
    _marker: PhantomData<T>,
}

impl<T, S> FramedRecvStream<T, S>
where
    S: AsyncRead,
    T: serde::de::DeserializeOwned,
{
    pub fn new(stream: S) -> Self {
        let mut codec = LengthDelimitedCodec::new();
        codec.set_max_frame_length(MAX_FRAME_SIZE);
        Self {
            read: FramedRead::new(stream, codec),
            _marker: PhantomData,
        }
    }
}

impl<T, S> Stream for FramedRecvStream<T, S>
where
    S: AsyncRead,
    T: serde::de::DeserializeOwned,
{
    type Item = Result<T, FrameError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let p = self.project();

        let bytes = ready!(p.read.poll_next(cx)?);

        match bytes {
            Some(v) => Poll::Ready(Some(
                bincode::deserialize(&v)
                    .map_err(|e| FrameError::DeserializePayload(e, type_name::<T>())),
            )),
            None => Poll::Ready(None),
        }
    }
}

/// Transport agnostic framed writer
#[pin_project]
pub struct FramedSendStream<T, S> {
    #[pin]
    write: FramedWrite<S, LengthDelimitedCodec>,
    _marker: PhantomData<T>,
}

impl<T, S> FramedSendStream<T, S>
where
    S: AsyncWrite,
    T: serde::Serialize,
{
    pub fn new(stream: S) -> Self {
        let mut codec = LengthDelimitedCodec::new();
        codec.set_max_frame_length(MAX_FRAME_SIZE);
        Self {
            write: FramedWrite::new(stream, codec),
            _marker: PhantomData,
        }
    }

    pub async fn send_bytes(&mut self, data: Bytes) -> Result<(), NetworkError>
    where
        S: Unpin,
    {
        self.write.send(data).await.map_err(Into::into)
    }
}

impl<T, S> Sink<&'_ T> for FramedSendStream<T, S>
where
    S: AsyncWrite,
    T: serde::Serialize,
{
    type Error = FrameError;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let p = self.project();

        p.write.poll_ready(cx).map_err(Into::into)
    }

    fn start_send(self: Pin<&mut Self>, item: &T) -> Result<(), Self::Error> {
        let p = self.project();

        let bytes = bincode::serialize(item)
            .map_err(|e| FrameError::SerializePayload(e, type_name::<T>()))?
            .into();
        p.write.start_send(bytes)?;

        Ok(())
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let p = self.project();

        p.write.poll_flush(cx).map_err(Into::into)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let p = self.project();

        p.write.poll_close(cx).map_err(Into::into)
    }
}

impl<T, S> Sink<T> for FramedSendStream<T, S>
where
    S: AsyncWrite,
    T: serde::Serialize,
{
    type Error = FrameError;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        <Self as Sink<&'_ T>>::poll_ready(self, cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        <Self as Sink<&'_ T>>::start_send(self, &item)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        <Self as Sink<&'_ T>>::poll_flush(self, cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        <Self as Sink<&'_ T>>::poll_close(self, cx)
    }
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("Failed to serialize payload of type {1}")]
    SerializePayload(#[source] bincode::Error, &'static str),
    #[error("Failed to deserialize payload of type {1}")]
    DeserializePayload(#[source] bincode::Error, &'static str),
    #[error("Invalid frame")]
    Io(#[from] std::io::Error),
}
