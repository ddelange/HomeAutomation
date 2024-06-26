use std::io::{BufRead, BufReader};
use std::net::{TcpStream, ToSocketAddrs};
use std::vec;

use protocol::{DecodeError, Msg};
use tokio::io::AsyncBufReadExt;
use tracing::trace;

pub enum SubMessage {
    Reading(protocol::Reading),
    ErrorReport(protocol::Error),
}

pub struct Subscriber {
    reader: BufReader<TcpStream>,
    buf: Vec<u8>,
    values: vec::IntoIter<SubMessage>,
}

impl Subscriber {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, SubscribeError> {
        let conn = TcpStream::connect(addr).map_err(SubscribeError::ConnFailed)?;
        let reader = BufReader::new(conn);
        let buf = Vec::new();

        Ok(Self {
            reader,
            buf,
            values: vec![].into_iter(),
        })
    }

    pub fn next(&mut self) -> Result<SubMessage, SubscribeError> {
        if let Some(val) = self.values.next() {
            return Ok(val);
        }

        self.buf.clear();
        let n_read = self
            .reader
            .read_until(0, &mut self.buf)
            .map_err(SubscribeError::ConnFailed)?;
        decode_buffer_and_return_first(n_read, &mut self.buf, &mut self.values)
    }
}

pub struct AsyncSubscriber {
    reader: tokio::io::BufReader<tokio::net::TcpStream>,
    buf: Vec<u8>,
    values: vec::IntoIter<SubMessage>,
}

impl AsyncSubscriber {
    pub async fn connect(addr: impl tokio::net::ToSocketAddrs) -> Result<Self, SubscribeError> {
        let conn = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(SubscribeError::ConnFailed)?;
        let reader = tokio::io::BufReader::new(conn);
        let buf = Vec::new();

        Ok(Self {
            reader,
            buf,
            values: vec![].into_iter(),
        })
    }

    pub async fn next(&mut self) -> Result<SubMessage, SubscribeError> {
        if let Some(val) = self.values.next() {
            return Ok(val);
        }

        let n_read = self
            .reader
            .read_until(0, &mut self.buf)
            .await
            .map_err(SubscribeError::ConnFailed)?;
        decode_buffer_and_return_first(n_read, &mut self.buf, &mut self.values)
    }
}

fn decode_buffer_and_return_first(
    n_read: usize,
    buf: &mut Vec<u8>,
    buffer: &mut vec::IntoIter<SubMessage>,
) -> Result<SubMessage, SubscribeError> {
    assert!(buffer.next().is_none());

    if n_read == 0 {
        return Err(SubscribeError::ConnEnded)?;
    }

    buf.resize(n_read, 0); // ensure stop delimiter in bytes
    let bytes = &mut buf[0..n_read];
    trace!("{:?}", &bytes);
    let decoded: Msg<50> = Msg::decode(bytes).map_err(SubscribeError::DecodeFailed)?;
    match decoded {
        Msg::Readings(readings) => {
            *buffer = readings
                .values
                .iter()
                .map(Clone::clone)
                .map(|v| SubMessage::Reading(v))
                .collect::<Vec<_>>()
                .into_iter();
        }
        Msg::ErrorReport(report) => {
            *buffer = vec![SubMessage::ErrorReport(report.error)].into_iter()
        }
    }

    Ok(buffer.next().expect("min values in sensormessage is one"))
}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("The connection to the subscribe server failed, error: {0}")]
    ConnFailed(std::io::Error),
    #[error("Could not decode message, is protocol lib up to date on server and client? Decoderror: {0:?}")]
    DecodeFailed(DecodeError),
    #[error("Connection ended")]
    ConnEnded,
}
