local __mod__ = (function()
local apply = (function(fun,a)
return fun(a)
end)

local add_ten = (function(a)
return a+10
end)

local bar = 100

local foo = apply(add_ten,bar)

return {
apply = apply,
add_ten = add_ten,
bar = bar,
foo = foo,
}
end)()
return __mod__