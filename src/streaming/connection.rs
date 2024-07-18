// A simple prefix-length framing abstraction for streaming protobufs
use anyhow::{Error, Result};
use bytes::BytesMut;
use log::error;
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Sender, oneshot},
};
use std::io::Cursor;
use log::info;

use crate::{config::READ_BUFFER_CAPACITY, request, Request};

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(READ_BUFFER_CAPACITY),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Request>> {
        loop {
            if let Some(req) = self.parse_frame()? {
                return Ok(Some(req));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    error!("Connection closed on client with non-empty buffer");
                    // return Err("connection reset by peer".into());
                    return Ok(None);
                }
            }
        }
    }

    fn parse_frame(&mut self) -> crate::Result<Option<Request>> {
        if (self.buffer.is_empty()) {
            return Ok(None);
        }
        let mut buf = Cursor::new(&self.buffer[..]);
        Ok(Some(Request::decode(&mut buf).unwrap()))
        // @todo krarpit complete
    } 
}