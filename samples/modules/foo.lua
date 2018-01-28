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

