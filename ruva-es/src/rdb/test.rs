use crate::aggregate::TAggregateMetadata;

use crate::{aggregate::TAggregateES, event::TEvent};

use ruva_core::prelude::serde::{Deserialize, Serialize};
use ruva_core::responses::ApplicationError;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(crate = "ruva_core::prelude::serde")]
pub struct Account {
	pub(crate) id: i64,
	pub(crate) name: String,
	pub(crate) hashed_password: String,
	pub(crate) version: i64,
	events: Vec<AccountEvent>,
}
impl Account {
	pub(crate) fn create_account(email: String, password: String) -> Self {
		let mut aggregate = Account {
			name: email.clone(),
			hashed_password: password.to_string() + "_hashed",
			..Default::default()
		};

		aggregate.raise_event(AccountEvent::AccountCreated {
			name: email,
			hashed_password: aggregate.hashed_password.clone(),
			id: aggregate.id,
		});
		aggregate
	}
	fn verify_password(&self, plain_text: &str) -> Result<(), Error> {
		//! for testing purpose
		if self.hashed_password == plain_text {
			return Err(Error);
		}
		Ok(())
	}
	pub(crate) fn sign_in(&mut self, email: String, password: String) -> Result<(), Error> {
		self.verify_password(&password)?;
		self.raise_event(AccountEvent::SignedIn { email, password });

		Ok(())
	}
}

impl TAggregateES for Account {
	type Event = AccountEvent;
	type Error = Error;
	type Command = AccountCommand;

	fn apply(&mut self, event: Self::Event) {
		match event {
			Self::Event::AccountCreated { id, name, hashed_password } => {
				*self = Account {
					id,
					name,
					hashed_password,
					..Default::default()
				}
			}
			Self::Event::SignedIn { .. } => {}
		}
	}

	fn raise_event(&mut self, event: Self::Event) {
		self.events.push(event)
	}
	fn events(&self) -> &Vec<Self::Event> {
		&self.events
	}
	fn handle(&mut self, cmd: Self::Command) -> Result<(), Self::Error> {
		match cmd {
			AccountCommand::CreateAccount { email, password } => {
				*self = Self::create_account(email, password);
				Ok(())
			}
			AccountCommand::SignInAccount { email, password } => self.sign_in(email, password),
		}
	}
}

impl TAggregateMetadata for Account {
	fn aggregate_type(&self) -> String {
		"Account".to_string()
	}
	fn aggregate_id(&self) -> String {
		self.id.to_string()
	}
	fn sequence(&self) -> i64 {
		self.version
	}
	fn set_sequence(&mut self, version: i64) {
		self.version = version
	}
}

#[derive(Deserialize, Clone)]
#[serde(crate = "ruva_core::prelude::serde")]
pub enum AccountCommand {
	CreateAccount { email: String, password: String },
	SignInAccount { email: String, password: String },
}

#[derive(Debug, Deserialize, PartialEq, Clone, Serialize)]
#[serde(crate = "ruva_core::prelude::serde")]
pub enum AccountEvent {
	AccountCreated { id: i64, name: String, hashed_password: String },
	SignedIn { email: String, password: String },
}

impl TEvent for AccountEvent {
	fn event_type(&self) -> String {
		let event_type_in_str = match self {
			Self::AccountCreated { .. } => "AccountCreated",
			Self::SignedIn { .. } => "SignIn",
		};
		event_type_in_str.to_string()
	}
	fn event_version(&self) -> String {
		"0.1".to_string()
	}

	fn aggregate_type(&self) -> String {
		"Account".to_string()
	}
}

#[derive(Debug)]
pub struct Error;
impl ApplicationError for Error {}

#[cfg(test)]
mod test_account {
	use crate::testing::TestFrameWork;

	use super::{Account, AccountCommand, AccountEvent};

	#[test]
	fn create_account() {
		let expected = AccountEvent::AccountCreated {
			id: 0,
			name: "test_email@mail.com".to_string(),
			hashed_password: "test_password_hashed".to_string(),
		};

		TestFrameWork::<Account>::new()
			.given_no_previous_events()
			.when(AccountCommand::CreateAccount {
				email: "test_email@mail.com".to_string(),
				password: "test_password".to_string(),
			})
			.then_expect_events(vec![expected]);
	}

	#[test]
	fn sign_in() {
		let expected = AccountEvent::SignedIn {
			email: "test_email@mail.com".to_string(),
			password: "test_password".to_string(),
		};

		TestFrameWork::<Account>::new()
			.given(vec![AccountEvent::AccountCreated {
				id: 0,
				name: "test_email@mail.com".to_string(),
				hashed_password: "test_password_hashed".to_string(),
			}])
			.when(AccountCommand::SignInAccount {
				email: "test_email@mail.com".to_string(),
				password: "test_password".to_string(),
			})
			.then_expect_events(vec![expected]);
	}

	#[test]
	fn sign_in_fail_case() {
		TestFrameWork::<Account>::new()
			.given(vec![AccountEvent::AccountCreated {
				id: 0,
				name: "test_email@mail.com".to_string(),
				hashed_password: "test_password_hashed".to_string(),
			}])
			.when(AccountCommand::SignInAccount {
				email: "test_email@mail.com".to_string(),
				password: "test_password_hashed".to_string(),
			})
			.then_expect_error_message("Error");
	}
}
