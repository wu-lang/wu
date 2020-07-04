return (function()
  local traits = require('traits')
  local foo = traits['foo']
  local Player = traits['Player']
  
  
  local lover = require('lover')
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