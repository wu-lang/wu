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


local a = (function(b,c,d,e,f)
do
end
end)

a("hey",bob["position"]["x"],bob["position"]["y"],bob["size"]["x"],bob["size"]["y"])

local b = bob["position"]["x"]

return {
point = point,
player = player,
bob = bob,
a = a,
b = b,
}
end)()
return __mod__