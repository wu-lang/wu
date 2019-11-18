return (function()
  local a = 100
  
  a = (a * 100)
  a = (a + 10)
  a = (a / 10)
  local c = "duggiduggiduk"
  
  c = (c .. "hey")
  c = (c .. a)
  return {
    a = a,
    c = c,
  }
end)()