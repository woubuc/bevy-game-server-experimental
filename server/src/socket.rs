use std::net::SocketAddr;
use std::thread;

use async_std::{channel, task};
use async_std::channel::{Receiver, Sender};
use async_std::net::{TcpListener, TcpStream};
use async_tungstenite::tungstenite::Message;
use futures::pin_mut;
use futures::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Entity, PrecisePosition};

/// Events sent by the server to clients
#[derive(Debug, Clone, Serialize)]
pub enum ServerToClientPacket {
	CounterChanged {
		name: String,
		value: f64,
	},
	PositionChanged(Entity, PrecisePosition),
}

/// Events received by the server from clients
#[derive(Debug, Clone, Deserialize)]
pub enum ClientToServerPacket {
	Hi(String),
}

/// Sender for outgoing packets
pub type SocketSender = Sender<ServerToClientPacket>;

/// Receiver for incoming packets
pub type SocketReceiver = Receiver<ClientToServerPacket>;

/// Spawns the websocket listener thread and returns a packet channel
///
/// The websocket thread uses an async runtime internally to manage two-way
/// communication across many connections.
pub fn spawn_socket_thread() -> (SocketSender, SocketReceiver) {
	let (to_client_sender, to_client_receiver) = channel::unbounded();
	let (to_server_sender, to_server_receiver) = channel::unbounded();

	// We're currently in a regular sync context so we first need to spawn a
	// new regular system thread in order to start the async runtime we need
	// for the websocket server.
	thread::spawn(|| {
		task::block_on(socket_thread(to_client_receiver, to_server_sender));
	});

	(to_client_sender, to_server_receiver)
}

/// Starts the socket listener
async fn socket_thread(
	to_client_receiver: Receiver<ServerToClientPacket>,
	to_server_sender: Sender<ClientToServerPacket>,
) {
	// TODO: we should probably make the listener address configurable at some point I think
	let listener = TcpListener::bind("127.0.0.1:3333").await.unwrap();
	println!("[Socket] Listening on port 3333 (yes that's hardcoded, deal with it)");

	while let Ok((stream, addr)) = listener.accept().await {
		task::spawn(handle_stream(
			stream,
			addr,
			to_client_receiver.clone(),
			to_server_sender.clone(),
		));
	}
}

/// Handles an incoming socket stream
async fn handle_stream(
	stream: TcpStream,
	addr: SocketAddr,
	mut to_client_receiver: Receiver<ServerToClientPacket>,
	to_server_sender: Sender<ClientToServerPacket>,
) {
	let ws = async_tungstenite::accept_async(stream).await.unwrap();
	println!("[Socket:{:?}] Connected!", addr);

	let (mut outgoing, mut incoming) = ws.split();

	let transmit_outgoing = async move {
		while let Some(packet) = to_client_receiver.next().await {
			if let Ok(json) = serde_json::to_string(&packet) {
				outgoing.send(Message::Text(json)).await.expect("Could not send outgoing packet");
			}
		}
	};

	let transmit_incoming = async move {
		while let Some(Ok(msg)) = incoming.next().await {
			if let Message::Text(json) = msg {
				if let Ok(packet) = serde_json::from_str::<ClientToServerPacket>(&json) {
					to_server_sender.send(packet).await.expect("Could not accept incoming packet");
				}
			}
		}
	};

	pin_mut!(transmit_outgoing, transmit_incoming);
	future::select(transmit_outgoing, transmit_incoming).await;

	println!("[Socket:{:?}] Disconnected :(", addr);
}
