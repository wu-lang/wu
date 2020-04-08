return (function()
  local Vector = {}
  
  
  Vector['move'] = function(self, dx, dy)
    self['x'] = (self['x'] + dx)
    self['y'] = (self['y'] + dy)
  end
  
  
  
  Vector['length'] = function(self)
    return (((self['x'] ^ 2) + (self['y'] ^ 2)) ^ 0.5)
  end
  
  
  
  local position = setmetatable({
    x = 100,
    y = 200,
  }, {__index=Vector})
  
  position['move'](position, 10, 10)
  return {
    Vector = Vector,
    Movable = Movable,
    position = position,
  }
end)()