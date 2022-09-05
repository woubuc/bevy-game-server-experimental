use std::net::SocketAddr;
use std::thread;

use async_std::channel::{Receiver, Sender};
use async_std::net::{TcpListener, TcpStream};
use async_std::{channel, task};
use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::{Message, WebSocket};
use async_tungstenite::WebSocketStream;
use futures::pin_mut;
use futures::prelude::*;
use serde::Deserialize;

use crate::auth::Auth;

use super::packets::{ClientToServerPacket, ServerToClientPacket};

/// Sender for outgoing packets
pub type SocketSender = Sender<ServerToClientPacket>;

/// Receiver for incoming packets
pub type SocketReceiver = Receiver<ClientToServerPacket>;

/// Spawns the websocket listener thread and returns a packet channel
///
/// The websocket thread uses an async runtime internally to manage two-way
/// communication across many connections.
pub fn spawn_socket_thread(auth: Auth) -> (SocketSender, SocketReceiver) {
	let (to_client_sender, to_client_receiver) = channel::unbounded();
	let (to_server_sender, to_server_receiver) = channel::unbounded();

	// We're currently in a regular sync context so we first need to spawn a
	// new regular system thread in order to start the async runtime we need
	// for the websocket server.
	thread::spawn(|| {
		task::block_on(socket_thread(to_client_receiver, to_server_sender, auth));
	});

	(to_client_sender, to_server_receiver)
}

/// Starts the socket listener
async fn socket_thread(
	to_client_receiver: Receiver<ServerToClientPacket>,
	to_server_sender: Sender<ClientToServerPacket>,
	auth: Auth,
) {
	// TODO: we should probably make the listener address configurable at some point I think
	let listener = TcpListener::bind("127.0.0.1:3333").await.unwrap();
	println!("[Socket] Listening on port 3333 (yes that's hardcoded, deal with it)");

	while let Ok((stream, addr)) = listener.accept().await {
		task::spawn(handle_unauthenticated_stream(
			stream,
			addr,
			to_client_receiver.clone(),
			to_server_sender.clone(),
			auth.clone(),
		));
	}
}

#[derive(Debug, Deserialize)]
struct SocketAuthPayload {
	token: String,
}

/// Handles an unauthenticated incoming socket stream
///
/// The first thing every opened socket should do is authenticate itself with
/// an auth token (see the [`crate::auth`] module for more details on authentication).
/// The socket may not do anything else until this auth token has been received
/// and validated, so the initial handler for each socket only listens for this
/// authentication message and validates the token.
///
/// If any incoming messages don't match the expected authentication flow, or
/// if the token is invalid, the connection is immediately closed.
///
/// Upon successful authentication, the socket stream is passed along to the
/// main handler [`handle_stream`].
async fn handle_unauthenticated_stream(
	stream: TcpStream,
	addr: SocketAddr,
	mut to_client_receiver: Receiver<ServerToClientPacket>,
	to_server_sender: Sender<ClientToServerPacket>,
	auth: Auth,
) {
	let mut ws = async_tungstenite::accept_async(stream).await.unwrap();
	println!("[Socket:{:?}] Connected!", addr);

	if let Some(Ok(Message::Text(json))) = ws.next().await {
		if let Ok(SocketAuthPayload { token }) = serde_json::from_str(&json) {
			if let Some(user) = auth.validate_token(&token).await {
				println!("[Socket:{:?}] Authenticated: {:?}", addr, user);
				handle_stream(ws, addr, to_client_receiver, to_server_sender).await;
				return;
			}
		}
	}

	// If no valid authentication arrived, close the socket
	println!("[Socket:{:?}] Invalid auth", addr);
	ws.close(Some(CloseFrame {
		code: CloseCode::Policy,
		reason: "token_invalid".into(),
	}))
	.await
	.expect("Could not close socket");
}

/// Handles an accepted socket stream after it has been authenticated with a token
async fn handle_stream(
	ws: WebSocketStream<TcpStream>,
	addr: SocketAddr,
	mut to_client_receiver: Receiver<ServerToClientPacket>,
	to_server_sender: Sender<ClientToServerPacket>,
) {
	let (mut outgoing, mut incoming) = ws.split();

	let transmit_outgoing = async move {
		while let Some(packet) = to_client_receiver.next().await {
			if let Ok(json) = serde_json::to_string(&packet) {
				outgoing
					.send(Message::Text(json))
					.await
					.expect("Could not send outgoing packet");
			}
		}
	};

	let transmit_incoming = async move {
		while let Some(Ok(msg)) = incoming.next().await {
			if let Message::Text(json) = msg {
				if let Ok(packet) = serde_json::from_str::<ClientToServerPacket>(&json) {
					to_server_sender
						.send(packet)
						.await
						.expect("Could not accept incoming packet");
				}
			}
		}
	};

	pin_mut!(transmit_outgoing, transmit_incoming);
	future::select(transmit_outgoing, transmit_incoming).await;

	println!("[Socket:{:?}] Disconnected", addr);
}
