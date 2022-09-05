use bevy::prelude::*;
use bevy::time::FixedTimestep;

use crate::auth::Auth;
use crate::{App, Counter, Entity, GameTickStage, Name, PrecisePosition, SystemStage};

pub use self::packets::*;
use self::socket_server::spawn_socket_thread;
pub use self::socket_server::{SocketReceiver, SocketSender};

mod packets;
mod socket_server;

/// How often changed entities should be sent to connected clients, in seconds
///
/// Data changes are only sent to the clients every so often to avoid flooding
/// the network or overloading low-powered clients.
pub const SOCKET_SEND_TIMESTEP: f64 = 1.0 / 10.0; // 10 times per second

#[derive(Debug, Clone, StageLabel)]
pub struct SocketProcessIncomingStage;

#[derive(Debug, Clone, StageLabel)]
pub struct SocketEmitOutgoingStage;

#[derive(Debug)]
pub struct SocketPlugin;

impl Plugin for SocketPlugin {
	fn build(&self, app: &mut App) {
		let auth = app.world.resource::<Auth>();

		// Keep track of the socket sender & the socket receiver as resources
		// in the Bevy world so we can use them in our systems.
		let (socket_sender, socket_receiver) = spawn_socket_thread(auth.clone());
		app.insert_resource(socket_sender);
		app.insert_resource(socket_receiver);

		// Define separate stages to handle socket data before & after the main game loop
		app.add_stage_before(
			GameTickStage,
			SocketProcessIncomingStage,
			SystemStage::parallel(),
		);
		app.add_stage_after(
			GameTickStage,
			SocketEmitOutgoingStage,
			SystemStage::parallel().with_run_criteria(FixedTimestep::step(SOCKET_SEND_TIMESTEP)),
		);

		app.add_system_to_stage(SocketProcessIncomingStage, socket_process_incoming);
		app.add_system_to_stage(SocketEmitOutgoingStage, socket_emit_counters);
		app.add_system_to_stage(SocketEmitOutgoingStage, socket_emit_positions);
	}
}

fn socket_process_incoming(receiver: Res<SocketReceiver>) {
	// We don't wait for new packets to arrive, we only process the packets that are already pending in the channel
	while let Ok(packet) = receiver.try_recv() {
		println!("[Game] Incoming packet: {:?}", packet);
	}
}

fn socket_emit_counters(
	query: Query<(&Name, &Counter), Changed<Counter>>,
	sender: Res<SocketSender>,
) {
	for (name, counter) in query.iter() {
		sender
			.send_blocking(ServerToClientPacket::CounterChanged {
				name: name.0.clone(),
				value: counter.0,
			})
			.unwrap();
	}
}

fn socket_emit_positions(
	query: Query<(Entity, &PrecisePosition), Changed<PrecisePosition>>,
	sender: Res<SocketSender>,
) {
	for (entity, pos) in query.iter() {
		sender
			.send_blocking(ServerToClientPacket::PositionChanged(entity, pos.clone()))
			.unwrap();
	}
}
