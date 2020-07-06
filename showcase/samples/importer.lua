return (function()
  local traits = require('showcase/samples/traits')
  local foo = traits['foo']
  local Player = traits['Player']
  
  
  local lover = require('showcase/samples/lover')
  local love = lover['love']
  
  
  local a = setmetatable({
    x = 100,
    y = 100,
  }, {__index=Player})
  
  a['move'](a, 10, 10)
  return {
    foo = foo,
    Player = Player,
    love = love,
    a = a,
  }
end)()