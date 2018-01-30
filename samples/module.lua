local __mod__ = (function()
local modules = require('samples/modules')
local test = (function()
local point = {
__construct__ = function(__constructor)
return {
x = __constructor.x,
y = __constructor.y,
}
end
}
local inner_test = (function()
local point_inner = {
__construct__ = function(__constructor)
return {
x = __constructor.x,
y = __constructor.y,
z = __constructor.z,
}
end
}
return {
point_inner = point_inner,
}
end)()
return {
point = point,
inner_test = inner_test,
}
end)()
local test_point = test["point"].__construct__({
x = 1,
y = 2,
z = 3,
})


return {
modules = modules,
test = test,
test_point = test_point,
}
end)()
return __mod__
