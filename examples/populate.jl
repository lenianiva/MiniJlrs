import MyModule

x = MyModule.Datum()
@show x.x
MyModule.populate!(x)
@show x.x
