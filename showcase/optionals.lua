return (function()
  
  local bee = 110
  
  local buzz = bee; assert(bee ~= nil)
  
  local foo = bee
  
  print(bee, buzz, foo, nil)
  return {
    print = print,
    bee = bee,
    buzz = buzz,
    foo = foo,
  }
end)()