module MyModule
#module Internal

import JlrsCore
using Base: Module
using JlrsCore.Wrap

print("The module is $(@__MODULE__)")
struct Datum
	x::Array{UInt8, 1}
end

path_lib = Base.abspath("target/debug/libminijlrs.$(Base.Libc.Libdl.dlext)")
include_dependency(path_lib)


@wrapmodule(path_lib, :mymodule_init_fn)

function __init__()
    @lock JlrsCore.package_lock JlrsCore.loaded_packages[:MyModule] = @__MODULE__
    @initjlrs
end

export Environment, Action, Agent, Trajectory, play

end

