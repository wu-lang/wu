local __mod__ = (function()
local something = {
__construct__ = function(__constructor)
return {
content = __constructor.content,
}
end
}
local make_something = (function(content)
return something.__construct__({
content = content,
})

end)

return {
something = something,
make_something = make_something,
}
end)()
return __mod__