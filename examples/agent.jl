using MyModule

a = Agent("test", (x) -> Action("action"))
play(a, UInt(1))
