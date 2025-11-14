import MyModule

e = MyModule.Zero()
a = MyModule.Agent("name", (_x) -> e)
MyModule.check(a)
@info "checkpoint 1"
r = MyModule.Runtime()
r2 = MyModule.Runtime()
@info "checkpoint 2"

for i in 1:5
    @info "i: $i"
end
