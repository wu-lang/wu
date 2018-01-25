local test = {
point = {
__construct__ = function(__constructor)
return {
x = __constructor.x,
y = __constructor.y,
}
end
},
inner_test = {
point_inner = {
__construct__ = function(__constructor)
return {
x = __constructor.x,
y = __constructor.y,
z = __constructor.z,
}
end
},
},
}
local test_point = test["point"].__construct__({
x = 1,
y = 2,
z = 3,
})


