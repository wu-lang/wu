return (function()
  local Foo = {}
  
  local grr = setmetatable({
    b = 100,
  }, {__index=Foo})
  
  grr['b'] = (grr['b'] + 10)
  local a = 100
  
  a = (a * 100)
  a = (a + 10)
  a = (a / 10)
  local c = "duggiduggiduk"
  
  c = (c .. "hey")
  c = (c .. a)
  return {
    Foo = Foo,
    grr = grr,
    a = a,
    c = c,
  }
end)()