use crate::aggregate::TAggregateMetadata;

use crate::{aggregate::TAggregateES, event::TEvent};

use ruva_core::prelude::{Deserialize, Serialize};
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
	pub(crate) fn create_account(cmd: CreateAccount) -> Self {
		let mut aggregate = Account {
			name: cmd.email.clone(),
			hashed_password: cmd.password + "_hashed",
			..Default::default()
		};

		aggregate.raise_event(AccountEvent::AccountCreated {
			name: cmd.email,
			hashed_password: aggregate.hashed_password.clone(),
			id: aggregate.id,
		});
		aggregate
	}
	fn verify_password(&self, plain_text: &str) -> Result<(), Error> {
		Ok(())
	}
	pub(crate) fn sign_in(&mut self, cmd: SignInAccount) -> Result<(), Error> {
		self.verify_password(&cmd.password)?;
		self.raise_event(AccountEvent::SignedIn {
			email: cmd.email,
			password: cmd.password,
		});
		self.version += 1;
		Ok(())
	}
}

impl TAggregateES for Account {
	type Event = AccountEvent;
	type Error = Error;

	fn apply(&mut self, event: Self::Event) {
		match event {
			Self::Event::AccountCreated { .. } => todo!(),
			Self::Event::SignedIn { .. } => todo!(),
		}
	}

	fn raise_event(&mut self, event: Self::Event) {
		self.events.push(event)
	}
	fn events(&self) -> &Vec<Self::Event> {
		&self.events
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
pub struct CreateAccount {
	pub email: String,
	pub password: String,
}

#[derive(Deserialize, Clone)]
#[serde(crate = "ruva_core::prelude::serde")]
pub struct SignInAccount {
	pub email: String,
	pub password: String,
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

	use ruva_core::prelude::tokio;
	use ruva_core::rdb::executor::SQLExecutor;

	use crate::{
		aggregate::TAggregateMetadata,
		event_store::TEventStore,
		rdb::{
			repository::SqlRepository,
			test::{Account, CreateAccount, SignInAccount},
		},
	};
	async fn clean_up() {
		dotenv::dotenv().ok();
		let executor = SQLExecutor::new();
		let _ = ruva_core::prelude::sqlx::query("TRUNCATE events,snapshots CASCADE").execute(executor.read().await.connection()).await;
	}

	#[tokio::test]
	async fn test_commit() {
		dotenv::dotenv().ok();
		let repo = SqlRepository::new(SQLExecutor::new());
		let aggregate = Account::create_account(CreateAccount {
			email: "test_email@mail.com".to_string(),
			password: "test_password".to_string(),
		});

		repo.commit(&aggregate).await.unwrap();
	}

	#[tokio::test]
	async fn test_load_aggregate() {
		clean_up().await;

		// given
		let repo = SqlRepository::new(SQLExecutor::new());
		let aggregate = Account::create_account(CreateAccount {
			email: "test_email@mail.com".to_string(),
			password: "test_password".to_string(),
		});
		repo.commit(&aggregate).await.unwrap();

		// when
		let account_aggregate = repo.load_aggregate(aggregate.id.to_string().as_str()).await.expect("Shouldn't fail!");

		//then
		assert_eq!(account_aggregate.sequence(), 1);
		assert_eq!(account_aggregate.name, "test_email@mail.com".to_string());
		assert_ne!(account_aggregate.hashed_password, "test_password".to_string());
	}

	#[tokio::test]
	async fn test_command_on_existing_aggregate() {
		clean_up().await;

		// given
		let repo = SqlRepository::new(SQLExecutor::new());
		let aggregate = Account::create_account(CreateAccount {
			email: "test_email@mail.com".to_string(),
			password: "test_password".to_string(),
		});

		repo.commit(&aggregate).await.unwrap();

		let mut account_aggregate = repo.load_aggregate(aggregate.id.to_string().as_str()).await.expect("Shouldn't fail!");

		// when
		account_aggregate
			.sign_in(SignInAccount {
				email: "test_email@mail.com".to_string(),
				password: "test_password".to_string(),
			})
			.unwrap();

		repo.commit(&account_aggregate).await.unwrap();

		// then
		let updated_account_aggregate = repo.load_aggregate(aggregate.id.to_string().as_str()).await.expect("Shouldn't fail!");

		assert_eq!(updated_account_aggregate.sequence(), 2);
	}
}
