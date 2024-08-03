// A simple prefix-length framing abstraction for streaming protobufs
use std::io::Cursor;
use std::io::ErrorKind;
use std::time::Duration;

use anyhow::Error;
use anyhow::Result;
use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use log::error;
use log::info;
use log::warn;
use prost::Message;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::time::sleep;

use crate::config::READ_BUFFER_CAPACITY;
use crate::request;
use crate::Request;

pub struct Connection {
    stream: TcpStream,
    // The buffer for reading protobuf "frames"
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(READ_BUFFER_CAPACITY),
        }
    }

    /// Read a single protobuf frame from the underlying stream.
    ///
    /// The function waits until it has retrieved enough data to parse a frame.
    /// Any data remaining in the read buffer after the frame has been parsed is
    /// kept there for the next call to `read_frame`. If the peer cleanly closes the
    /// connection `Ok(None)` is returned. If the connection is not cleanly closed
    /// (i.e buffer is non-empty) an Err is returned.
    pub async fn read_frame(&mut self) -> Result<Option<Request>> {
        loop {
            // Attempts to parse a frame from the buffered data if enough data
            // has been read.
            if let Some(req) = self.parse_frame().await? {
                return Ok(Some(req));
            }

            // There isn't enough data in the buffer, so attempt to read more
            // data.
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote has closed the connection. If the buffer is non-empty,
                // that indicates the peer closed the connection in the middle of
                // sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    error!("Connection closed on client with non-empty buffer");
                    return Err(Error::msg("Connection unexpectedly closed by peer"));
                }
            }
        }
    }

    /// Write given buffer of data and flushes any buffered writes. Returns an `Err` if
    /// write or flush fails.
    pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
        self.stream
            .write(buf)
            .await
            .map(|_| ())
            .map_err(|err| err.into())
            .and(self.stream.flush().await.map_err(|err| err.into()))
    }

    /// Tries to parse a frame from the buffer. If the buffer contains enough
    /// data, the frame is returned and the data removed from the buffer. If not
    /// enough data has been buffered yet, `Ok(None)` is returned. If the
    /// buffered data does not represent a valid frame, or read fails for some
    /// reason, an `Err` is returned.
    async fn parse_frame(&mut self) -> crate::Result<Option<Request>> {
        if (self.buffer.is_empty()) {
            return Ok(None);
        }
        let mut buf = Cursor::new(&self.buffer[..]);
        let len = buf.get_u64();
        info!("Length of recieved frame is {:?}", len);
        let mut data = vec![0u8; len.try_into().unwrap()];
        match buf.read_exact(&mut data).await {
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => {
                    warn!(
                        "Not enough bytes read in buffer; Failed with error={:?}",
                        err
                    );
                    return Ok(None);
                },
                _ => return Err(err.into()),
            },
            _ => (),
        }
        // Drop all read data
        self.buffer
            .advance(<u64 as TryInto<usize>>::try_into(len).unwrap() + 8);
        let parsed_frame = Request::decode(Bytes::from(data))?;
        Ok(Some(parsed_frame))
    }
}
