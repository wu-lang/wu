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
speed = __constructor.speed,
}
end
}
local bob = player.__construct__({
position = point.__construct__({
x = 100,
y = 100,
})
,
speed = 200,
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

local c = #({
[0] = 1,
[1] = 2,
[2] = 3,
[3] = 4,
})

a("hey",bob["position"]["x"],bob["position"]["y"],bob["size"]["x"],bob["size"]["y"])

local b = bob["position"]["x"]

return {
point = point,
player = player,
bob = bob,
a = a,
c = c,
b = b,
}
end)()
return __mod__