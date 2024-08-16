#![allow(unused)]
pub trait TMyTrait<T, U> {
	fn my_method5(&self, a: T, b: U);
	fn my_method(&self) -> impl std::future::Future<Output = ()>;
}
struct A;

#[ruva::inject]
impl TMyTrait<i32, i32> for A {
	fn my_method5(&self, _a: i32, _b: i32) {}
	async fn my_method(&self) {}
}

struct B;

#[ruva::inject]
impl TMyTrait<i32, i32> for B {
	fn my_method5(&self, _a: i32, _b: i32) {}
	async fn my_method(&self) {}
}

#[test]
fn test_resolve() {
	let a = A;

	let b = (a, 1);
	b.my_method5(1, 1);

	let x = (B, A);
	x.my_method5(1, 2);

	let c = (A, 1, 2);
	c.my_method5(1, 2);

	let d = (A, 1, 2, "a".to_string());
	d.my_method5(1, 2);

	let e = (A, 1, 2, "a".to_string(), 1.0);
	e.my_method5(1, 2);
}

#[test]
fn test_message_handler_without_generic() {
	//WHEN
	#[ruva::message_handler]
	fn my_handler(a: String, b: i32, c: i32) -> (i32, i32) {
		(b, c)
	}

	//THEN input tuplified
	_ = __my_handler(1.to_string(), (1, 2));
	_ = my_handler(1.to_string(), 1, 2);
}

#[test]
fn test_message_handler_with_generic() {
	//GIVEN

	//WHEN
	#[ruva::message_handler]
	async fn my_handler_with_generic<T: TMyTrait<i32, i32>>(a: String, b: i32, c: T) {
		c.my_method5(b, b);
		(c, b).my_method5(b, b);
	}

	//THEN input tuplified
	_ = __my_handler_with_generic(1.to_string(), (1, A));
	_ = __my_handler_with_generic(1.to_string(), (1, B));

	_ = my_handler_with_generic(1.to_string(), 1, A);

	//adding `ruva::message_handler` means that tuplified type implement the generic too.
	fn func_take_my_trait<T: TMyTrait<i32, i32>>((a, b): (T, i32)) {
		(a, b).my_method5(b, b);
	}
}

#[test]
fn test_tuplified_trait() {
	trait TTrait {
		fn my_method(&self);
	}
	struct A;
	impl TTrait for A {
		fn my_method(&self) {
			println!("Hello");
		}
	}

	impl<T: TTrait, U> TTrait for (T, U) {
		fn my_method(&self) {
			self.0.my_method()
		}
	}

	fn func_take_my_trait<T: TTrait>((a, b): (T, i32)) {
		(a, b).my_method();
	}
	fn func_take_my_trait2<T: TTrait>((a, b): (i32, T)) {
		(b, a).my_method();
	}

	let a = || {
		struct B;
		impl TTrait for B {
			fn my_method(&self) {
				println!("Hello");
			}
		}
		func_take_my_trait((B, 1));
	};
}

#[test]
fn test_tuplified_generic_trait() {
	trait TTrait<T, U> {
		fn m1(&self, a: T);
		fn m2(&self, b: U);
	}
	struct A;

	#[ruva::inject]
	impl TTrait<i32, String> for A {
		fn m1(&self, _a: i32) {}
		fn m2(&self, _b: String) {}
	}

	fn func_take_my_trait<T: TTrait<i32, String>>((a, b): (T, i32)) {
		(a, b).m1(b);
	}
	fn func_take_my_trait2<T: TTrait<i32, String>>((a, b): (i32, T)) {
		(b, a).m2("a".into());
	}
}
