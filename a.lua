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
a = self.point_inner.__construct__({
x = 10,
y = 10,
z = 10,
})

,
},
}
local test_point = test["point"].__construct__({
x = 100,
y = 100,
})
