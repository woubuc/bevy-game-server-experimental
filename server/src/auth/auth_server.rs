use std::thread;

use async_std::task;
use serde::{Deserialize, Serialize};
use tide::prelude::*;
use tide::security::CorsMiddleware;
use tide::security::Origin;
use tide::Body;

use super::Auth;
use super::UserHandle;

/// Spawns the auth server thread and returns the associated auth API
///
/// The auth server thread uses an async runtime internally to improve
/// throughput of the HTTP server.
pub fn spawn_auth_thread() -> Auth {
	let auth = Auth::new();
	let thread_auth = auth.clone();

	// We're currently in a regular sync context so we first need to spawn a
	// new regular system thread in order to start the async runtime we need
	// for the auth HTTP server.
	thread::spawn(move || {
		task::block_on(auth_thread(thread_auth));
	});

	auth
}

/// Starts the auth HTTP server
async fn auth_thread(auth: Auth) {
	let mut app = tide::with_state(auth);
	app.with(CorsMiddleware::new().allow_origin(Origin::from("*")));

	app.at("/login").post(login);

	// TODO: another addr that probably should be made configurable
	let mut listener = app
		.bind("127.0.0.1:3000")
		.await
		.expect("Could not bind auth server");

	println!("[Auth] Listening on port 3000");
	listener.accept().await.expect("Error in auth server");
}

type Request = tide::Request<Auth>;

#[derive(Debug, Deserialize)]
struct LoginBody {
	email: String,
	password: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
enum LoginResult {
	Invalid,
	Success { token: String },
}

async fn login(mut req: Request) -> tide::Result<Body> {
	let body = req.body_json::<LoginBody>().await?;
	println!("[Auth] Login: {}", body.email);
	// TODO: actually implement this

	let token = req.state().generate_token(UserHandle { id: 1 }).await;
	Body::from_json(&LoginResult::Success { token })
}
