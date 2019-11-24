return (function()
  local Vector = {}
  
  Vector['seub'] = function(self, other)
    return setmetatable({
      x = (self['x'] + other['x']),
    }, {__index=Self})
  end
  
  Vector['add'] = function(a)
    return setmetatable({
      x = 10,
    }, {__index=Self})
  end
  
  
  local b = Vector['add'](10)
  
  return {
    Vector = Vector,
    b = b,
  }
end)()