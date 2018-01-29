local __mod__ = (function()
local foo = require('foo')
local something = foo.something
local make_something = foo.make_something

local booty = something.__construct__({
content = "yes yes yes",
})


return {
foo = foo,
booty = booty,
}
end)()
return __mod__