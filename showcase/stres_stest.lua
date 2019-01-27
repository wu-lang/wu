return (function()
  local Foo = {}
  
  local Bar = {}
  
  local bob = setmetatable({
    a = setmetatable({
      x = 100,
      y = 200,
    }, {__index=Foo}),
  }, {__index=Bar})
  
  local system = (function()
    
    return {
      print = print,
    }
  end)()
  
  system['print'](bob['a']['x'], bob['a']['y'])
  local cover = Bar
  
  return {
    Foo = Foo,
    Bar = Bar,
    bob = bob,
    system = system,
    cover = cover,
  }
end)()