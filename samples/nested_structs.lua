local __mod__ = (function()
local point = {
__construct__ = function(__constructor)
return {
x = __constructor.x,
y = __constructor.y,
}
end
}
local player = {
__construct__ = function(__constructor)
return {
position = __constructor.position,
size = __constructor.size,
}
end
}
local bob = player.__construct__({
position = point.__construct__({
x = 100,
y = 100,
})
,
size = point.__construct__({
x = 32,
y = 32,
})
,
})


local b = bob["position"]["x"]

return {
point = point,
player = player,
bob = bob,
b = b,
}
end)()
return __mod__