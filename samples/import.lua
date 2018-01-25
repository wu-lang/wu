local test = {
yes = {
__construct__ = function(__constructor)
return {
content = __constructor.content,
}
end
},
no = {
__construct__ = function(__constructor)
return {
content = __constructor.content,
}
end
},
}
local yes = test.yes
local no = test.no

local something = yes.__construct__({
content = "hello world",
})


local idk = no.__construct__({
content = "yes ok",
})


