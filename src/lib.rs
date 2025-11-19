use jlrs::{
	data::managed::{
		string::StringRet,
		value::{
			tracked::Tracked, typed::{TypedValue, TypedValueRet}, ValueRet
		},
	},
	memory::gc::{gc_safe, gc_unsafe},
	prelude::*,
	weak_handle,
};

#[repr(C)]
#[derive(Clone, Debug, OpaqueType)]
struct Environment {
	s: String,
}

impl Environment {
	fn new() -> TypedValueRet<Self> {
		let x = Self {
			s: "[]".to_string(),
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
struct Action {
	s: String,
}
impl Action {
	fn new(s: JuliaString<'_>) -> JlrsResult<TypedValueRet<Self>> {
		let x = Self {
			s: s.as_str()?.to_string(),
		};
		match weak_handle!() {
			Ok(handle) => Ok(TypedValue::new(handle, x).leak()),
			Err(_) => panic!("Not called from Julia"),
		}
	}
}

#[derive(Debug, ForeignType)]
pub struct Agent {
	name: String,
	#[jlrs(mark)]
	callback: ValueRet,
}
unsafe impl Send for Agent {}
unsafe impl Sync for Agent {}
impl Clone for Agent
{
	fn clone(&self) -> Self {
		match weak_handle!() {
			Ok(handle) => {
				Self {
					name: self.name.clone(),
					callback: unsafe { self.callback.root(&handle) }.leak()
				}
			}
			Err(e) => panic!("Not called from Julia"),
		}
	}
}
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
			Err(e) => panic!("Not called from Julia"),
		}
	}
	fn act(&self, env: Environment) -> Action {
		unsafe {
			gc_unsafe(|handle| {
				handle.local_scope::<_, 3>(|mut frame| {
					let callback = self.callback.as_value();
					let env = Value::new(&mut frame, env);
					let result = callback.call(&mut frame, [env]).expect("Error 1");
					result.leak().as_value().unbox::<Action>().unwrap()
					//let action: Tracked<'_, '_, '_, Action> = result.track_shared().expect("track");
					//action.clone()
				})
			})
		}
	}
}
#[repr(C)]
#[derive(Clone, Debug, OpaqueType)]
struct Trajectory {
	actions: Vec<String>,
}

fn play_loop(agent: Agent, steps: usize) -> Trajectory {
	let env = Environment { s: "".to_string() };
	let mut actions = vec![];
	for i in 0..steps {
		eprintln!("call {i}");
		let Action { s } = agent.act(env.clone());
		actions.push(s);
	}
	Trajectory { actions }
}
fn play(agent: TypedValue<'_, '_, Agent>, steps: usize) -> JlrsResult<TypedValueRet<Trajectory>> {
	let agent_r = agent.unbox::<Agent>()?;
	match weak_handle!() {
		Ok(handle) => handle.with_local_scope::<_, 2>(|handle, mut frame| {
			let t = unsafe { gc_safe(|| play_loop(agent_r, steps)) };
			Ok(TypedValue::new(handle, t).leak())
		}),
		Err(e) => panic!("Not called from Julia"),
	}
}

julia_module! {
	become mymodule_init_fn;

	struct Environment;
	struct Action;
	in Action fn new(
		name: JuliaString,
	) -> JlrsResult<TypedValueRet<Action>> as Action;
	struct Agent;
	in Agent fn new(
		name: JuliaString,
		callback: Value<'_, 'static>,
	) -> JlrsResult<TypedValueRet<Agent>> as Agent;

	struct Trajectory;
	fn play(agent: TypedValue<'_, '_, Agent>, steps: usize) -> JlrsResult<TypedValueRet<Trajectory>>;
}
