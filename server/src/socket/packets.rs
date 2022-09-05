use bevy::prelude::Entity;
use serde::{Deserialize, Serialize};

use crate::PrecisePosition;

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
