use std::{
    io::{self, Read},
    pin::Pin,
    sync::Once,
    task::{Context, Poll},
};

use log::LevelFilter;
use shared::logger::initialize_logger;
use tokio::io::{AsyncBufRead, AsyncRead, ReadBuf};

static INIT: Once = Once::new();

pub fn setup_logger() {
    INIT.call_once(|| {
        initialize_logger(LevelFilter::Trace);
    });
}

#[allow(dead_code)]
pub(crate) struct ErrorReader;

impl AsyncRead for ErrorReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Poll::Ready(Err(io::Error::other("simulated error".to_string())))
    }
}

impl AsyncBufRead for ErrorReader {
    fn poll_fill_buf(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        Poll::Ready(Err(io::Error::other("simulated error".to_string())))
    }
    fn consume(self: Pin<&mut Self>, _amt: usize) {}
}

impl Read for ErrorReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("simulated error".to_string()))
    }
}
