use bevy::prelude::*;

use crate::socket::{ServerToClientPacket, SocketReceiver, SocketSender, spawn_socket_thread};

mod socket;

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

		// Post-tick systems
		.add_system_to_stage(CoreStage::PostUpdate, socket_emit_counters)

		.run();
}


#[derive(Debug, Component)]
struct Name(String);

#[derive(Debug, Component)]
struct Counter(f64);

fn setup(mut cmd: Commands) {
	cmd.spawn()
		.insert(Counter(0.0))
		.insert(Name("foo".to_string()));

	cmd.spawn()
		.insert(Counter(0.0))
		.insert(Name("bar".to_string()));
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


fn update_counters(mut query: Query<&mut Counter>, time: Res<Time>) {
	let seconds = time.seconds_since_startup().floor();
	for mut counter in query.iter_mut() {
		if counter.0 != seconds {
			counter.0 = seconds;
		}
	}
}
