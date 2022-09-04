use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Component)]
pub struct Character {
	pub avatar: String,
}

#[derive(Debug, Component)]
pub struct PlayerControlled {
	pub player_id: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
	pub x: u64,
	pub y: u64,
	pub z: u16,
}

#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct PrecisePosition {
	pub position: Position,
	pub precise_x: u8,
	pub precise_y: u8,
	pub precise_z: u8,
}
