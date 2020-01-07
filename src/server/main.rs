use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use evolutionmp::setup_logger;
use evolutionmp::network::{Message, MessageReceiver, MessageSender};
use tokio::net::TcpListener;
use tokio::prelude::Async;
use futures::Future;
use futures::sync::mpsc::{channel, Receiver, Sender};
use log::{info, error, warn};
use winapi::_core::task::{Context, Poll};
use winapi::_core::pin::Pin;
use tokio::prelude::future::FutureResult;
use tokio::sync::mpsc::{Receiver, Sender};

pub(crate) struct TaskExecutor {
    pending_tasks: VecDeque<Receiver<(SocketAddr, TaskResult)>>
}

impl TaskExecutor {
    pub fn new() -> TaskExecutor {
        TaskExecutor {
            pending_tasks: VecDeque::new()
        }
    }

    pub fn submit(&mut self, task: Task) {
        let (s, r) = channel::<(SocketAddr, TaskResult)>();

        tokio::executor::spawn(task.and_then(move |result| {
            match s.send(result) {
                Err(_) => error!("Failed to return task result"),
                _ => {}
            }
            Ok(())
        }));

        futures::task::current().notify();

        self.pending_tasks.push_back(r);
    }
}

#[tokio::main]
pub async fn main() {
    let debug = std::env::args().any(|a|a == "--debug");
    setup_logger("server", debug);

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), evolutionmp::network::PORT);
    let listener = TcpListener::bind(&address).await?;

    info!(target: evolutionmp::LOG_ROOT, "Listening on :{}", address.port());

    let server = Server {
        active_connections: HashMap::new(),
        task_executor: TaskExecutor::new(),
        listener
    };
    tokio::run(server);
}

pub enum ConnectionStage {
    LoggingIn {
        socialclub: String,
        pid: u32
    }
}

pub(crate) struct Connection {
    receiver: Receiver<Message>,
    sender: Sender<Message>,
    start_time: Instant,
    last_active: Instant
}

pub enum Task {

}

impl std::future::Future for Task {
    type Output = TaskResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}

pub enum TaskResult {
    SendMessage {
        message: Message
    }
}

impl Connection {
    fn process_task_result(&mut self, result: TaskResult) {
        match result {
            TaskResult::SendMessage { message } => {
                self.send(message);
            }
        }
    }

    fn update_stage(&mut self, stage: ConnectionStage) {
        self.stage = stage;
        match &self.stage {
            ConnectionStage::LoggingIn { .. } => {

            },
            _ => {}
        }
    }

    fn send(&mut self, message: Message) {
        self.sender.try_send(message).expect("Failed to send message");
    }
}

struct Server {
    active_connections: HashMap<SocketAddr, Connection>,
    task_executor: TaskExecutor,
    listener: TcpListener
}

impl Server {
    async fn tick(&mut self) {
        loop {
            while let Ok(Async::Ready(task)) = self.task_executor.await {
                let (peer, result) = task.expect("Task channel closed");
                if let Some(connection) = self.active_connections.get_mut(&peer) {
                    connection.process_task_result(result);
                }
            }

            let mut disconnected_peers = Vec::new();
            let mut tasks = Vec::new();

            for (peer, connection) in &mut self.active_connections {
                while let Ok(Async::Ready(Some(message))) = connection.receiver.poll() {
                    if let Message::Disconnect = message {
                        disconnected_peers.push(peer.clone());
                    } else {
                        if let Some(task) = process_message(peer, connection, message) {
                            tasks.push(task);
                        }
                    }
                }
            }

            for task in tasks {
                self.task_executor.submit(task);
            }

            for peer in disconnected_peers {
                self.active_connections.remove(&peer);
                info!(target: evolutionmp::LOG_ROOT, "{} lost connection: Disconnected", peer);
            }

            while let (stream, peer) = self.listener.poll_accept().map_err(|_|()).await {
                stream.set_keepalive(Some(Duration::from_secs(15)))
                    .expect("Failed to set keep alive interval");
                stream.set_recv_buffer_size(evolutionmp::network::MAX_PACKET_SIZE)
                    .expect("Failed to set recv buffer size");
                stream.set_send_buffer_size(evolutionmp::network::MAX_PACKET_SIZE)
                    .expect("Failed to set send buffer size");
                stream.set_nodelay(false)
                    .expect("Failed to set nodelay");

                let stream = Arc::new(Mutex::new(stream));
                info!("Incoming connection from {}", peer);

                if self.active_connections.contains_key(&peer) {
                    warn!("Already connected");
                } else {
                    let start_time = Instant::now();

                    let (mut inbound_s, inbound_r) = channel::<Message>(5);
                    let (outbound_s, outbound_r) = channel::<Message>(5);

                    let connection = Connection {
                        receiver: inbound_r,
                        sender: outbound_s,
                        last_active: start_time.clone(),
                        start_time
                    };
                    self.active_connections.insert(peer, connection);

                    tokio::spawn(MessageReceiver::new(stream.clone(), move |m| {
                        match inbound_s.try_send(m) {
                            Err(e) => {
                                if !e.is_disconnected() {
                                    error!("Error receiving message: {:?}", e);
                                }
                            },
                            _ => {}
                        }
                    }).join(MessageSender::new(stream, outbound_r)).map(|_|()));

                    futures::task::current().notify();
                }
            }
        }
    }
}

fn process_message(peer: &SocketAddr, connection: &mut Connection, message: Message) -> Option<Task> {
    match message {
        Message::Connect { socialclub, pid } => {
            connection.info = Some(ConnectionStage::LoggingIn {
                socialclub, pid
            });
        },
        other => unimplemented!("No handler for message {:?}", other)
    }
    None
}