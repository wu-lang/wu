return (function()
  local Player = {}
  
  
  Player['move'] = function(self, dx, dy)
    self['x'] = (self['x'] + dx)
    self['y'] = (self['y'] + dy)
  end
  
  
  
  local niels = setmetatable({
    x = 100,
    y = 200,
  }, {__index=Player})
  
  niels['name'] = "boss man"
  return {
    Player = Player,
    Moving = Moving,
    niels = niels,
  }
end)()