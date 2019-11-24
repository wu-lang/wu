return (function()
  local Vector = {}
  
  Vector['length'] = function(self)
    return (((self['x'] ^ 2) + ((self['y'] ^ 2) + (self['z'] ^ 2))) ^ 0.5)
  end
  
  Vector['normalize'] = function(self)
    local len = self['length'](self)
    self['x'] = (self['x'] / len)
    self['y'] = (self['y'] / len)
    self['z'] = (self['z'] / len)
  end
  
  
  local len = function(a) return #a end
  
  local println = print
  
  function normalize_all(...)
    local bulk = {...}
    local i = 1
    while (i < len(bulk)) do
      repeat
      local vector = bulk[i]; assert(bulk[i] ~= nil, "can't unwrap 'nil'")
      
      println(vector['length'](vector))
      vector['normalize'](vector)
      println(vector['length'](vector))
      i = (i + 1)
      until true
    end
  end
  local a = setmetatable({
    x = 100,
    y = 200,
    z = 300,
  }, {__index=Vector})
  
  local b = setmetatable({
    x = 200,
    y = 300,
    z = 400,
  }, {__index=Vector})
  
  normalize_all(a, b)
  return {
    Vector = Vector,
    len = len,
    println = println,
    normalize_all = normalize_all,
    a = a,
    b = b,
  }
end)()