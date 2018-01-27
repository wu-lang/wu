local test = (function()
local yes = {
__construct__ = function(__constructor)
return {
content = __constructor.content,
}
end
}
local a = yes.__construct__({
content = "heyheyheyyy",
})


local no = {
__construct__ = function(__constructor)
return {
content = __constructor.content,
}
end
}
return {
yes = yes,
a = a,
no = no,
}
end)()
local yes = test.yes
local no = test.no

local something = yes.__construct__({
content = "hello world",
})


local b = test["a"]

local idk = no.__construct__({
content = "yes ok",
})
