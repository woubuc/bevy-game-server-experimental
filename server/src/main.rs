use bevy::prelude::*;
use bevy::time::FixedTimestep;

use crate::character::{Character, PlayerControlled};
use crate::character::{Position, PrecisePosition};
use crate::socket::SocketPlugin;

mod socket;
mod character;

/// Fixed timestep for the server
pub const FIXED_TIMESTEP: f64 = 1.0 / 60.0; // 60 ticks per second

/// The main stage of the game
///
/// All game logic should run in the game tick stage to make effective use of
/// Bevy's multithreading benefits.
#[derive(Debug, Clone, StageLabel)]
pub struct GameTickStage;

fn main() {
	App::new()
		// We only use the minimal plugins because on a server we have no use for windowing, graphics, audio, etc
		.add_plugins(MinimalPlugins)

		// Main game logic
		.add_startup_system(setup)
		.add_stage(GameTickStage, SystemStage::parallel().with_run_criteria(FixedTimestep::step(FIXED_TIMESTEP)))
		.add_system_to_stage(GameTickStage, update_counters)
		.add_system_to_stage(GameTickStage, move_characters)

		// Plugins
		.add_plugin(SocketPlugin)

		.run();
}


#[derive(Debug, Component)]
struct Name(pub String);

impl Name {
	pub fn new(name: &str) -> Self {
		Name(name.to_owned())
	}
}

#[derive(Debug, Component)]
struct Counter(pub f64);

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
