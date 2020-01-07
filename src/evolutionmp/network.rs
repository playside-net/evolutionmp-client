use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::error::Error;
use std::io::Error as IoError;
use std::collections::{VecDeque, HashMap};
use std::time::Instant;

use tokio::net::{UdpSocket, UdpFramed, TcpStream};
use tokio::codec::{Encoder, Decoder};
use tokio::prelude::*;

use serde_derive::{Serialize, Deserialize};
use log::{debug, error};
use colored::Colorize;
use bincode::{self, Error as BinError};
use std::sync::{Arc, Mutex};
use byteorder::{ReadBytesExt, WriteBytesExt, BE};
use tokio::sync::mpsc::Receiver;
use std::future::Future;

pub const IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const PORT: u16 = 2809;
pub const MAX_MTU: usize = 1400;
pub const MAX_PACKET_SIZE: usize = 512 * 1024;

pub struct MessageSender {
    stream: Arc<Mutex<TcpStream>>,
    input: Receiver<Message>,
    current_bytes: usize,
    total_bytes: usize,
    data: Vec<u8>
}

impl MessageSender {
    pub fn new(stream: Arc<Mutex<TcpStream>>, input: Receiver<Message>) -> MessageSender {
        MessageSender {
            stream,
            input,
            data: Vec::new(),
            current_bytes: 0,
            total_bytes: 0
        }
    }

    pub async fn run(&mut self) {
        loop {
            if self.total_bytes == 0 {
                let message: Option<Message> = self.input.poll().await?;
                if let Some(message) = message {
                    let data = bincode::serialize(&message)
                        .expect("Serialization failed");

                    if data.len() > MAX_PACKET_SIZE {
                        panic!("Message is larger than maximal packet size ({} > {})", data.len(), MAX_PACKET_SIZE);
                    }
                    self.current_bytes = 0;
                    self.total_bytes = data.len();
                    self.data = data;
                    debug!("{} {:?} ({} bytes)", "Sending".blue(), message, self.total_bytes);
                } else {
                    return Ok(Async::NotReady);
                }
            }
            let mut stream = self.stream.lock().unwrap();
            if self.current_bytes == 0 {
                let mut tmp: [u8; 4] = [0; 4];
                (&mut tmp[..]).write_u32::<BE>(self.total_bytes as u32).unwrap();
                stream.poll_write(&tmp).map_err(|e| {
                    error!("Message header writing failed: {:?}", e);
                }).await?;
            }
            while self.current_bytes < self.total_bytes {
                let a = stream.poll_write(&self.data).map_err(|e| {
                    error!("Message chunk writing failed: {:?}", e);
                }).await?;
                self.current_bytes += a;
                debug!("Written {} bytes ({}/{})", a, self.current_bytes, self.total_bytes);
            }
            self.current_bytes = 0;
            self.total_bytes = 0;
            self.data.clear();
        }
    }
}

pub struct MessageReceiver<F> where F: FnMut(Message) {
    stream: Arc<Mutex<TcpStream>>,
    output: F,
    total_bytes: usize,
    data: Vec<u8>
}

impl<F> MessageReceiver<F> where F: FnMut(Message) {
    pub fn new(stream: Arc<Mutex<TcpStream>>, output: F) -> MessageReceiver<F> {
        MessageReceiver { stream, output, total_bytes: 0, data: Vec::new() }
    }

    pub async fn run(&mut self) {
        loop {
            let a: Option<String> = Some(String::from("b"));
            let mut stream = self.stream.lock().unwrap();
            if self.total_bytes == 0 {
                let mut a: [u8; 4] = [0; 4];
                match stream.poll_read(&mut a) {
                    Err(e) => {
                        error!("Error reading packet header: {:?}", e);
                        (self.output)(Message::Disconnect);
                        return Ok(Async::Ready(()));
                    },
                    o => o
                }.map_err(|_|()).await?;
                self.total_bytes = (&a[..]).read_u32::<BE>().unwrap() as usize;
            }

            if self.total_bytes as usize > MAX_PACKET_SIZE {
                error!("Inbound is larger than allowed ({} > {})", self.total_bytes, MAX_PACKET_SIZE);
                return Ok(Async::Ready(()));
            } else {
                let mut tmp: [u8; MAX_MTU] = [0; MAX_MTU];
                while self.data.len() < self.total_bytes {
                    let rem = self.total_bytes - self.data.len();
                    let to = rem.min(MAX_MTU);
                    let a = stream.poll_read(&mut tmp[0..to]).map_err(|e|error!("Error reading packet chunk: {:?}", e)).await?;
                    if a > 0 {
                        self.data.extend_from_slice(&tmp[0..a]);
                        debug!("Read {} bytes ({}/{})", a, self.data.len(), self.total_bytes);
                    } else {
                        break;
                    }
                }
                if self.data.is_empty() {
                    (self.output)(Message::Disconnect);
                    return Ok(Async::Ready(()))
                }
                let message = bincode::deserialize::<Message>(&self.data)
                    .expect("Deserialization failed");
                debug!("{} {:?}", "Received".red(), message);
                (self.output)(message);
                self.total_bytes = 0;
                self.data.clear();
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Disconnect,
    Connect {
        socialclub: String,
        pid: u32
    }
}