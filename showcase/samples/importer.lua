return (function()
  local traits = require('showcase/samples/traits')
  foo = traits['foo']
  Player = traits['Player']
  
  
  local lover = require('showcase/samples/lover')
  love = lover['love']
  
  
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