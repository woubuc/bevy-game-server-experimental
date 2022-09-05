use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::utils::{HashMap, Instant};
use rand::{thread_rng, Rng};

use super::UserHandle;

/// Maximum age of a token
///
/// If a token has not been used in this time, is is invalidated and deleted.
const TOKEN_MAX_AGE: Duration = Duration::from_secs(60); // 1 minute

/// Data associated with an auth token
#[derive(Debug)]
struct TokenData {
	user: UserHandle,
	added_at: Instant,
}

/// Public API for the authentication service
///
/// `Auth` is thread-safe and can be cloned and passed along to where it is needed.
#[derive(Debug, Clone)]
pub struct Auth {
	tokens: Arc<Mutex<HashMap<String, TokenData>>>,
}

impl Auth {
	/// Creates an empty Auth struct without any data
	pub(super) fn new() -> Self {
		Auth {
			tokens: Arc::new(Mutex::new(HashMap::default())),
		}
	}

	/// Generates a random token and associates it with the given user
	pub(super) async fn generate_token(&self, user: UserHandle) -> String {
		let token: String = thread_rng()
			.sample_iter(&rand::distributions::Alphanumeric)
			.take(64)
			.map(char::from)
			.collect();

		self.tokens
			.lock()
			.expect("Could not lock auth mutex")
			.insert(
				token.clone(),
				TokenData {
					user,
					added_at: Instant::now(),
				},
			);

		token
	}

	/// Validate a token
	///
	/// Tokens are single-use. When a token is successfully validated is it
	/// removed from the list of valid tokens and the user must create a new
	/// login token in order to authenticate again.
	pub async fn validate_token(&self, token: &str) -> Option<UserHandle> {
		self.tokens
			.lock()
			.expect("Could not lock auth mutex")
			.remove(token)
			.filter(|data| data.added_at.elapsed() <= TOKEN_MAX_AGE)
			.map(|data| data.user)
	}

	/// Removes all expired tokens from memory
	///
	/// This should be called periodically to prevent the list from growing indefinitely
	pub(super) fn cleanup(&self) {
		self.tokens
			.lock()
			.expect("Could not lock auth mutex")
			.retain(|_, v| v.added_at.elapsed() <= TOKEN_MAX_AGE);
	}
}
