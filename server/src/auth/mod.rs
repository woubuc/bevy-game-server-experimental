use crate::auth::auth_server::spawn_auth_thread;
use crate::FixedTimestep;
use bevy::prelude::*;

pub use self::auth::Auth;

mod auth;
mod auth_server;

/// How often invalid tokens should be removed from memory, in seconds
const AUTH_CLEANUP_TIME: f64 = 10.0;

pub struct AuthPlugin;

impl Plugin for AuthPlugin {
	fn build(&self, app: &mut App) {
		let auth = spawn_auth_thread();
		app.insert_resource(auth);

		// The auth cleanup system only runs every so often
		app.add_system_set_to_stage(
			CoreStage::Last,
			SystemSet::new()
				.with_run_criteria(FixedTimestep::step(AUTH_CLEANUP_TIME))
				.with_system(|auth: Res<Auth>| auth.cleanup()),
		);
	}
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct UserHandle {
	id: usize,
}
