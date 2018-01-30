local __mod__ = (function()
local test = require('./samples/modules/test')
return {
test = test,
}
end)()
return __mod__