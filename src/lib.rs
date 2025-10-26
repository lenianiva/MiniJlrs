use std::sync::Arc;

use jlrs::{
	data::{
		managed::{
			array::{ArrayRet, TypedRankedArrayRet, TypedVectorRet},
			string::StringRet,
			value::typed::{TypedValue, TypedValueRet},
		},
	},
	prelude::*,
	weak_handle,
};

#[repr(C)]
#[derive(Clone, Debug, OpaqueType)]
struct Expr {
	s: String,
}

impl Expr {
	fn new_zero() -> TypedValueRet<Self> {
		let x = Self {
			s: "Zero".to_string(),
		};
		match weak_handle!() {
			Ok(handle) => TypedValue::new(handle, x).leak(),
			Err(_) => panic!("Not called from Julia"),
		}
	}
	fn to_string(&self) -> StringRet {
		match weak_handle!() {
			Err(_) => panic!("Not called from Julia"),
			Ok(handle) => handle
				.local_scope::<_, 1>(|mut frame| JuliaString::new(&mut frame, self.s.clone()).leak()),
		}
	}
}

#[repr(C)]
#[derive(Clone, Debug, OpaqueType)]
struct ExprR(Arc<String>);

fn create_array_of_expr() -> TypedVectorRet<ExprR> {
	match weak_handle!() {
		Err(e) => panic!("not called from julia"),
		Ok(handle) => handle.local_scope::<_, 12>(|mut frame| {
			let n = 10;
			let mut v = TypedVector::<ExprR>::new(&mut frame, n).expect("Incompatible size and layout");
			let mut accessor = unsafe { v.indeterminate_data_mut() };
			for i in 1..n {
				let expr =
					"test".to_string();
				let value = Value::new(&mut frame, ExprR(Arc::new(expr)));
				accessor
					.set_value(&mut frame, i, value)
					.expect("Index out of bounds, which is impossible")
					.expect("Caught exception");
			}
			v.leak()
		}),
	}
}

#[repr(C)]
#[derive(
	Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg, CCallReturn,
)]
#[cfg_attr(not(test), jlrs(julia_type = "MyModule.Datum"))]
#[cfg_attr(test, jlrs(julia_type = "Main.Datum"))]
pub struct Datum<'scope, 'data> {
	//pub x: Option<TypedRankedArrayRef<'scope, 'data, u8, 1>>,
	pub x: Option<WeakTypedArray<'scope, 'data, u8>>,
}

fn generate() -> Datum<'static, 'static> {
	match weak_handle!() {
		Err(_) => panic!("Not called from Julia"),
		Ok(handle) => handle.local_scope::<_, 1>(|mut frame| {
			let x = TypedArray::new(&mut frame, [3]).expect("E1").leak();
			Datum { x: Some(x) }
		}),
	}
}

julia_module! {
	become mymodule_init_fn;

	fn generate() -> Datum<'static, 'static>;

	struct Expr as Expression;
	struct ExprR as ExprR;
	in Expr fn new_zero() -> TypedValueRet<Expr> as Zero;

	#[untracked_self]
	in Expr fn to_string(&self) -> StringRet as Base.string;
}

#[cfg(test)]
mod tests {
	use super::*;
	use jlrs::convert::unbox::Unbox;

	#[test]
	fn expr_zero() {
		let handle = Builder::new().start_local().expect("cannot init Julia");

		handle.local_scope::<_, 0>(|frame| {
			unsafe {
				Value::eval_string(
					&frame,
					"
				struct Datum
					sort::Array{UInt8, 1}
				end
				",
				)
				.expect("Adding structure failed");
				mymodule_init_fn(Module::main(&frame), /*must be set to 1*/ 1)
			};

			frame.local_scope::<_, 2>(|mut frame| {
				let v = Expr::new_zero();
				let string_fn = Module::base(&frame).global(&mut frame, "string").unwrap();
				let result =
					unsafe { string_fn.call(&mut frame, [v.as_value()]) }.expect("to string failed");
				let result_s = unsafe { String::unbox(result) }.expect("Invalid string");
				assert_eq!(result_s, "Zero");
			})
		})
	}

	#[test]
	fn expr_array() {
		let handle = Builder::new().start_local().expect("cannot init Julia");

		handle.local_scope::<_, 0>(|frame| {
			unsafe {
				Value::eval_string(
					&frame,
					"
				struct Datum
					sort::Array{UInt8, 1}
				end
				",
				)
				.expect("Adding structure failed");
				mymodule_init_fn(Module::main(&frame), /*must be set to 1*/ 1)
			};

			create_array_of_expr();
		})
	}
}
