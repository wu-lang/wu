local __mod__ = (function()


local point = {
__construct__ = function(__constructor)
return {
x = __constructor.x,
y = __constructor.y,
}
end
}
local position = point.__construct__({
x = 100,
y = 100,
})


love["update"] = (function(dt)
do
position["x"] = position["x"]+dt*10

position["y"] = position["y"]+dt*10*1.5

end
end)
love["draw"] = (function()
return love["graphics"]["rectangle"]("fill",position["x"],position["y"],200,200)
end)
return {
point = point,
position = position,
}
end)()
return __mod__