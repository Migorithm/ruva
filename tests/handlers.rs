use ruva::serde_json;

#[test]
fn test_injectable_trait() {
	#[allow(unused)]
	#[ruva::injectable]
	pub trait TMyTrait<T, U> {
		fn my_method(&self);
		fn my_method2(&mut self);
		fn my_method3(&self, a: i32);
		fn my_method4(self, a: U);
		fn my_method5(&self, a: T, b: U);
	}

	struct A;
	impl TMyTrait<String, i32> for A {
		fn my_method(&self) {}
		fn my_method2(&mut self) {}
		fn my_method3(&self, _a: i32) {}
		fn my_method4(self, _a: i32) {}

		fn my_method5(&self, _a: String, _b: i32) {
			todo!()
		}
	}
	fn func_take_my_trait<T: TMyTrait<String, i32>>(_t: T) {
		let method_info: &str = T::__RV_M_SIGNATURE;
		assert_eq!(
			method_info.replace(' ', ""),
			"fnmy_method(&self)▁DLM▁fnmy_method2(&mutself)▁DLM▁fnmy_method3(&self,a:i32)▁DLM▁fnmy_method4(self,a:U)▁DLM▁fnmy_method5(&self,a:T,b:U)"
		);

		let generic_mapper: &str = T::__RV_G_MAP;
		let mapper = serde_json::from_str::<serde_json::Value>(generic_mapper).unwrap();

		let method_applied_to_first_generic = &mapper["0"];
		assert_eq!(*method_applied_to_first_generic, serde_json::json!([["my_method5", 1]]));

		let method_applied_to_second_generic = &mapper["1"];
		assert_eq!(*method_applied_to_second_generic, serde_json::json!([["my_method4", 1], ["my_method5", 2]]));
	}

	func_take_my_trait(A);
}
