local __mod__ = (function()
local inside_bar = {
__construct__ = function(__constructor)
return {
yes = __constructor.yes,
}
end
}
return {
inside_bar = inside_bar,
}
end)()
return __mod__