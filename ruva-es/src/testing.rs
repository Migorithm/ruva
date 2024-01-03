use std::marker::PhantomData;

use crate::aggregate::TAggregateES;

#[derive(Default)]
pub struct TestFrameWork<A: TAggregateES>(PhantomData<A>);

impl<A> TestFrameWork<A>
where
	A: TAggregateES,
{
	pub fn new() -> Self {
		Self(Default::default())
	}
	pub fn given_no_previous_events(self) -> AggregateTestExecutor<A> {
		AggregateTestExecutor::new(Vec::new())
	}

	pub fn given(self, events: Vec<A::Event>) -> AggregateTestExecutor<A> {
		AggregateTestExecutor::new(events)
	}
}

pub struct AggregateTestExecutor<A>
where
	A: TAggregateES,
{
	events: Vec<A::Event>,
}
impl<A> AggregateTestExecutor<A>
where
	A: TAggregateES,
{
	pub fn new(events: Vec<A::Event>) -> Self {
		Self { events }
	}

	pub fn when(self, command: A::Command) -> AggregateResultValidator<A> {
		let mut aggregate = A::default();
		self.events.into_iter().for_each(|event| aggregate.apply(event));
		let res = aggregate.handle(command);

		AggregateResultValidator::new(res.map(|_| aggregate.events().clone()))
	}
}

pub struct AggregateResultValidator<A>
where
	A: TAggregateES,
{
	result: Result<Vec<A::Event>, A::Error>,
}

impl<A: TAggregateES> AggregateResultValidator<A> {
	pub fn then_expect_events(self, expected_events: Vec<A::Event>) {
		let events = match self.result {
			Ok(expected_events) => expected_events,
			Err(err) => {
				panic!("expected success, received aggregate error: '{:?}'", err);
			}
		};
		assert_eq!(events, expected_events);
	}

	pub fn then_expect_error_message(self, error_message: &str) {
		match self.result {
			Ok(events) => {
				panic!("expected error, received events: '{:?}'", events);
			}
			Err(err) => assert_eq!(format!("{:?}", err), error_message.to_string()),
		};
	}

	pub fn get_result(self) -> Result<Vec<A::Event>, A::Error> {
		self.result
	}

	pub(crate) fn new(result: Result<Vec<A::Event>, A::Error>) -> Self {
		Self { result }
	}
}
