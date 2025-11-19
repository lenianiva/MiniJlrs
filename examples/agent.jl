using MyModule

@info "create agent"
a = Agent("test", (x) -> Action("action"))
@info "play"
play(a, UInt(3))
