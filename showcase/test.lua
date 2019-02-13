return (function()
  
  
  
  local Table = {}
  
  Table['set'] = function(self, key, value)
    return rawset(self, key, value)
  end
  
  Table['get'] = function(self, key)
    return rawget(self, key)
  end
  
  
  local a = setmetatable({
  }, {__index=Table})
  
  a['set'](a, 1, 1)
  print(a['get'](a, 1))
  return {
    print = print,
    rawget = rawget,
    rawset = rawset,
    Table = Table,
    a = a,
  }
end)()