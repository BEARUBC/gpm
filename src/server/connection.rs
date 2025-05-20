// Wrapper around a TCP connection to provide a simple prefix-length framing abstraction for streaming protobufs
use crate::Request;
use crate::config::Config;
use anyhow::Error;
use anyhow::Result;
use bytes::Buf;
use bytes::Bytes;
use bytes::BytesMut;
use log::error;
use log::warn;
use prost::Message;
use std::io::Cursor;
use std::io::ErrorKind;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Represents a TCP connection
pub struct Connection {
    stream: TcpStream,
    /// The buffer for reading protobuf "frames"
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let server_config = Config::global()
            .server
            .as_ref()
            .expect("Global config must include server config");
        Connection {
            stream,
            buffer: BytesMut::with_capacity(server_config.read_buffer_capacity_in_bytes as usize),
        }
    }

    /// Read a single protobuf frame from the underlying stream. If the peer cleanly closes
    /// the connection `Ok(None)` is returned. If the connection is not cleanly closed
    /// (i.e buffer is non-empty) an Err is returned.
    pub async fn read_frame(&mut self) -> Result<Option<Request>> {
        loop {
            if let Some(req) = self.parse_frame().await? {
                return Ok(Some(req));
            }

            // Not enough data in the buffer, so attempt to read more data
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
    async fn parse_frame(&mut self) -> Result<Option<Request>> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        let mut buf = Cursor::new(&self.buffer[..]);
        let len: usize = buf
            .get_u64()
            .try_into()
            .expect("Data length should fit into `usize`");
        let mut data = vec![0u8; len];
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
        let server_config = Config::global()
            .server
            .as_ref()
            .expect("Expected server config to be defined");
        // Drop all read data
        self.buffer
            .advance(len + server_config.frame_prefix_length_in_bytes as usize);
        let parsed_frame = Request::decode(Bytes::from(data))?;
        Ok(Some(parsed_frame))
    }
}
