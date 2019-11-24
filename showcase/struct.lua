return (function()
  local Foo = {}
  
  
  Foo['foobar'] = function(a)
    return nil
  end
  
  
  Foo['boo'] = function()
    return setmetatable({
    }, {__index=Self})
  end
  
  
  Foo['foo'] = function()
    return setmetatable({
    }, {__index=Self})
  end
  
  Foo['bob'] = function(self, a)
    return setmetatable({
    }, {__index=Self})
  end
  
  
  local fo = Foo['foo']()
  
  return {
    Foo = Foo,
    Bar = Bar,
    fo = fo,
  }
end)()