use bevy::prelude::*;
use bevy::time::FixedTimestep;

use crate::character::{Character, PlayerControlled};
use crate::character::{Position, PrecisePosition};
use crate::socket::{ServerToClientPacket, SocketReceiver, SocketSender, spawn_socket_thread};

mod socket;
mod character;

/// How often changed entities should be sent to connected clients, in seconds
///
/// Data changes are only sent to the clients every so often to avoid flooding
/// the network or overloading low-powered clients.
pub const SOCKET_SEND_TIMESTEP: f64 = 1.0 / 10.0; // 10 updates per second

fn main() {
	let (socket_sender, socket_receiver) = spawn_socket_thread();

	App::new()
		// We only use the minimal plugins because on a server we have no use for windowing, graphics, audio, etc
		.add_plugins(MinimalPlugins)

		// Keep track of the socket sender & the socket receiver as resources
		// in the Bevy world so we can use them in our systems.
		.insert_resource(socket_sender)
		.insert_resource(socket_receiver)

		// Startup
		.add_startup_system(setup)

		// Pre-tick systems
		.add_system_to_stage(CoreStage::PreUpdate, socket_process_incoming)

		// Tick systems
		.add_system(update_counters)
		.add_system(move_characters)

		// Post-tick systems
		.add_system_set_to_stage(CoreStage::PostUpdate, SystemSet::new()
			.with_run_criteria(FixedTimestep::step(SOCKET_SEND_TIMESTEP))
			.with_system(socket_emit_counters)
			.with_system(socket_emit_positions),
		)

		.run();
}


#[derive(Debug, Component)]
struct Name(String);

impl Name {
	pub fn new(name: &str) -> Self {
		Name(name.to_owned())
	}
}

#[derive(Debug, Component)]
struct Counter(f64);

fn setup(mut cmd: Commands) {
	cmd.spawn()
		.insert(Counter(0.0))
		.insert(Name("foo".to_string()));

	cmd.spawn()
		.insert(Counter(0.0))
		.insert(Name("bar".to_string()));

	cmd.spawn()
		.insert(Name::new("John Smith"))
		.insert(Character {
			avatar: "idk".to_owned(),
		})
		.insert(PlayerControlled {
			player_id: 1,
		})
		.insert(PrecisePosition {
			position: Position {
				x: 12,
				y: 16,
				z: 2,
			},
			precise_x: 0,
			precise_y: 0,
			precise_z: 0,
		});
}


fn socket_process_incoming(receiver: Res<SocketReceiver>) {
	// We don't wait for new packets to arrive, we only process the packets that are already pending in the channel
	while let Ok(packet) = receiver.try_recv() {
		println!("[Game] Incoming packet: {:?}", packet);
	}
}

fn socket_emit_counters(query: Query<(&Name, &Counter), Changed<Counter>>, sender: Res<SocketSender>) {
	for (name, counter) in query.iter() {
		sender.send_blocking(ServerToClientPacket::CounterChanged { name: name.0.clone(), value: counter.0 }).unwrap();
	}
}

fn socket_emit_positions(query: Query<(Entity, &PrecisePosition), Changed<PrecisePosition>>, sender: Res<SocketSender>) {
	for (entity, pos) in query.iter() {
		sender.send_blocking(ServerToClientPacket::PositionChanged(entity, pos.clone())).unwrap();
	}
}


fn update_counters(mut query: Query<&mut Counter>, time: Res<Time>) {
	let seconds = time.seconds_since_startup().floor();
	for mut counter in query.iter_mut() {
		if counter.0 != seconds {
			counter.0 = seconds;
		}
	}
}

fn move_characters(mut query: Query<&mut PrecisePosition, With<Character>>) {
	for mut pos in query.iter_mut() {
		pos.precise_x = pos.precise_x.wrapping_add(1);
		if pos.precise_x == 0 {
			pos.position.x += 1;
		}
	}
}
