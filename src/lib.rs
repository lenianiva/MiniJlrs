use std::borrow::BorrowMut;

use jlrs::{
	data::{
		managed::{
			array::{ArrayRet, TypedRankedArrayRet},
			ccall_ref::CCallRef,
			string::StringRet,
			value::typed::{TypedValue, TypedValueRef, TypedValueRet},
		},
		types::foreign_type::OpaqueType,
	},
	error::{JlrsError, JuliaResultExt},
	prelude::*,
	weak_handle,
};

#[repr(C)]
#[derive(Clone, Debug)]
struct Expr {
	s: String,
}
unsafe impl OpaqueType for Expr {}
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
#[derive(
	Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg, CCallReturn,
)]
#[cfg_attr(not(test), jlrs(julia_type = "MyModule.Datum"))]
#[cfg_attr(test, jlrs(julia_type = "Main.Datum"))]
pub struct Datum<'scope, 'data> {
	pub x: Option<TypedRankedArrayRef<'scope, 'data, u8, 1>>,
}

fn generate() -> Datum<'static, 'static> {
	match weak_handle!() {
		Err(_) => panic!("Not called from Julia"),
		Ok(handle) => handle.local_scope::<_, 1>(|mut frame| {
			let x = TypedRankedArray::new(&mut frame, (3,)).expect("E1").leak();
			Datum { x: Some(x) }
		}),
	}
}
fn populate(mut d: TypedValue<Datum<'static, 'static>>) -> JlrsResult<()> {
	let mut inner = unsafe { d.track_exclusive() }?;
	let z = match weak_handle!() {
		Err(_) => panic!("Not called from Julia"),
		Ok(handle) => handle.local_scope::<_, 1>(|mut frame| {
			let data = vec![1u8, 2u8, 4u8];
			let x = TypedRankedArray::from_vec(&mut frame, data, (3,))
				.expect("result 1")
				.leak()
				.expect("e2");
			x
		}),
	};
	inner.x = Some(z);
	Ok(())
}

julia_module! {
	become mymodule_init_fn;

	struct Expr as Expression;
	in Expr fn new_zero() -> TypedValueRet<Expr> as Zero;

	#[untracked_self]
	in Expr fn to_string(&self) -> StringRet as Base.string;

	fn generate() -> Datum<'static, 'static>;
	fn populate(d: TypedValue<Datum<'static, 'static>>) -> JlrsResult<()> as populate!;
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
					unsafe { string_fn.call1(&mut frame, v.as_value()) }.expect("to string failed");
				let result_s = unsafe { String::unbox(result) }.expect("Invalid string");
				assert_eq!(result_s, "Zero");
			})
		})
	}
}
