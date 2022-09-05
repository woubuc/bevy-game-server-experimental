//! This module contains the authentication server for the game.
//!
//! ### Authentication Flow
//! Player authentication happens through a straightforward REST API that runs
//! alongside the main game server. To connect to the game server via a socket,
//! players first need to log in through the API and acquire an authentication
//! token.
//!
//! 1. Client logs in through the REST API and creates an authenticated HTTP
//!    session that can persist over time
//! 2. Client requests a one-time authentication token through the REST API,
//!    based on their previously authenticated HTTP session
//! 3. Client connects to the socket server (see the [`socket`] module
//!    for more information about the socket server) and authenticates the
//!    connection using the one-time authentication token.
//!
//! ###### Note
//! Authentication is not implemented as of yet. The single temporary route
//! `/login` generates a random token for testing during early development.
//!
//! ### Auth Provider
//! The [`Auth`] resource (provided by the [`AuthPlugin`]) provides a way to
//! validate authentication tokens from other modules.

use bevy::prelude::*;

use crate::FixedTimestep;
use crate::socket;

pub use self::auth_provider::Auth;
use self::auth_server::spawn_auth_thread;

mod auth_provider;
mod auth_server;

/// How often invalid tokens should be removed from memory, in seconds
const AUTH_CLEANUP_TIME: f64 = 10.0;

/// Authentication server plugin
///
/// This plugin manages the HTTP authentication server and provides the
/// [`Auth`] resource.
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


/// References a
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct UserHandle {
	id: usize,
}
