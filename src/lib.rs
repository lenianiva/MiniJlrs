use std::{sync::Arc, thread::JoinHandle};

use jlrs::{
	data::managed::{
		string::StringRet,
		value::{
			typed::{TypedValue, TypedValueRet},
			ValueRet,
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

pub fn create_compat_runtime() -> std::io::Result<tokio::runtime::Runtime> {
	tokio::runtime::Builder::new_current_thread()
		.enable_time()
		//.worker_threads(threads)
		//.on_thread_start(|| {
		//	assert!(!ADOPTED.get());
		//	// Extracted from `jlrs`'s `MtHandle`
		//	let mut ptls = unsafe { jlrs_get_ptls_states() };
		//	if ptls.is_null() {
		//		let pgcstack = unsafe { jl_adopt_thread() };
		//		ptls = unsafe { jlrs_ptls_from_gcstack(pgcstack) };
		//	}
		//	unsafe { jlrs_gc_safe_enter(ptls) };
		//	ADOPTED.set(true);
		//	unsafe { jl_enter_threaded_region() };
		//})
		//.on_thread_stop(|| {
		//	let ptls = unsafe { jlrs_get_ptls_states() };
		//	unsafe { jlrs_gc_safe_enter(ptls) };
		//})
		.build()
}
#[repr(C)]
#[derive(Debug, OpaqueType)]
pub struct Runtime {
	pub runtime: Arc<tokio::runtime::Runtime>,
	pub join_handle: JoinHandle<()>,
}
impl Runtime {
	fn new() -> TypedValueRet<Self> {
		let join_handle = std::thread::Builder::new()
			.spawn(|| {
				std::thread::sleep_ms(10000);
			})
			.unwrap();
		match weak_handle!() {
			Ok(handle) => {
				let x = Self {
					runtime: Arc::new(create_compat_runtime().unwrap()),
					join_handle,
				};
				TypedValue::new(handle, x).leak()
			}
			Err(_) => panic!("Not called from Julia"),
		}
	}
}
#[repr(C)]
#[derive(Debug, ForeignType)]
pub struct Agent {
	name: String,
	/**
	The callback function, which generates [`TacticQueue`] given a goal.
	 */
	#[jlrs(mark)]
	callback: ValueRet,
}
unsafe impl Send for Agent {}
unsafe impl Sync for Agent {}
impl Agent {
	pub fn new(name: JuliaString, callback: Value<'_, 'static>) -> JlrsResult<TypedValueRet<Self>> {
		let name = name.as_str()?.to_string();
		match weak_handle!() {
			Ok(handle) => {
				let data = Self {
					name,
					callback: callback.leak(),
				};
				Ok(TypedValue::new(handle, data).leak())
			}
			Err(_e) => unreachable!(),
		}
	}
	pub fn check(&self) -> JlrsResult<()> {
		match weak_handle!() {
			Ok(handle) => handle.local_scope::<_, 3>(|mut frame| {
				let callback = unsafe { self.callback.as_value() };
				let o = Value::new(&mut frame, 123);
				let result = unsafe { callback.call(&mut frame, [o]) }.expect("Error 1");
				let _e = result.unbox::<Expr>()?;
				Ok(())
			}),
			Err(_e) => unreachable!(),
		}
	}
}

julia_module! {
	become mymodule_init_fn;

	struct Expr as Expression;
	in Expr fn new_zero() -> TypedValueRet<Expr> as Zero;

	#[untracked_self]
	in Expr fn to_string(&self) -> StringRet as Base.string;

	struct Runtime;

	in Runtime fn new() -> TypedValueRet<Runtime> as Runtime;

	struct Agent;
	in Agent fn new(
		name: JuliaString,
		callback: Value<'_, 'static>,
	) -> JlrsResult<TypedValueRet<Agent>> as Agent;
	in Agent fn check(&self) -> JlrsResult<()>;
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
