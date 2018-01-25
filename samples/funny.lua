local apply = (function(fun,a)
return fun(a)
end)

local add_ten = (function(a)
return a+10
end)

local bar = 100

local foo = apply(add_ten,bar)

