use crate::runtime::{Script, ScriptEnv, Runtime};
//use crate::network::{Message, MessageSender, MessageReceiver};
use tokio::net::TcpStream;
use tokio::prelude::*;
use std::net::SocketAddr;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, channel as std_channel};
use tokio::sync::mpsc::{Sender, channel};

pub fn init(runtime: &mut Runtime) {
    runtime.register_script("network", ScriptNetwork {
        receiver: None,
        sender: None
    });
}

pub struct ScriptNetwork {
    receiver: Option<Receiver<Message>>,
    sender: Option<Sender<Message>>
}

impl Script for ScriptNetwork {
    fn prepare(&mut self, mut env: ScriptEnv) {
        let address = SocketAddr::new(crate::network::IP, crate::network::PORT);
        //self.connect(address);
    }
}

impl ScriptNetwork {
    fn connect(&mut self, address: SocketAddr) {
        let (inbound_s, inbound_r)
            = std_channel::<Message>();
        let (outbound_s, outbound_r)
            = channel::<Message>(5);

        self.receiver = Some(inbound_r);
        self.sender = Some(outbound_s);

        tokio::spawn(TcpStream::connect(&address).then(|connection| {
            let connection = connection.expect("Server connection failed");
            connection.set_keepalive(Some(Duration::from_secs(10)))
                .expect("Failed to set keep alive interval");
            connection.set_recv_buffer_size(crate::network::MAX_PACKET_SIZE)
                .expect("Failed to set recv buffer size");
            connection.set_send_buffer_size(crate::network::MAX_PACKET_SIZE)
                .expect("Failed to set send buffer size");
            connection.set_nodelay(false)
                .expect("Failed to set nodelay");

            let connection = Arc::new(Mutex::new(connection));

            MessageSender::new(connection.clone(), outbound_r).join(
                MessageReceiver::new(connection.clone(), move |m| {
                    inbound_s.send(m).expect("Error receiving message")
                })
            )
        }).map_err(|e| {
            crate::error!("Connection error: {:?}", e);
            crate::error_message!("Ошибка соединения", "Не удаётся соединиться с сайтом\nПроверьте интернет-соединение и перезапустите\nлаунчер!");
        }).map(|_|()));
    }
}
