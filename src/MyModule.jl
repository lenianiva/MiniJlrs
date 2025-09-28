module MyModule
using JlrsCore.Wrap

print("The module is $(@__MODULE__)")
struct Datum
	x::Array{UInt8, 1}
end

path_lib = Base.abspath("target/debug/libminijlrs.so")
include_dependency(path_lib)
@wrapmodule(path_lib, :mymodule_init_fn)

function __init__()
    @initjlrs
end

end
