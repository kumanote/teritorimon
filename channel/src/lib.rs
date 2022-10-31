use futures::{
    channel::mpsc,
    sink::Sink,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct Sender<T> {
    inner: mpsc::Sender<T>,
}

impl<T> Sink<T> for Sender<T> {
    type Error = mpsc::SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (*self).inner.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, msg: T) -> Result<(), Self::Error> {
        (*self).inner.start_send(msg)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl<T> Sender<T> {
    pub fn try_send(&mut self, msg: T) -> Result<(), mpsc::SendError> {
        (*self)
            .inner
            .try_send(msg)
            .map_err(mpsc::TrySendError::into_send_error)
    }
}

#[derive(Debug)]
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> FusedStream for Receiver<T>
where
    T: std::fmt::Debug,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

pub fn new<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (sender, receiver) = mpsc::channel(size);
    (Sender { inner: sender }, Receiver { inner: receiver })
}
