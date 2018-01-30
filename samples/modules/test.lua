local __mod__ = (function()
local foo = require('././samples/modules/foo')
local something = foo.something
local make_something = foo.make_something

local bar = require('././samples/modules/bar')
local booty = something.__construct__({
content = "yes yes yes",
})


local bass = bar["inside_bar"].__construct__({
yes = 123214.123,
})


return {
foo = foo,
bar = bar,
booty = booty,
bass = bass,
}
end)()
return __mod__