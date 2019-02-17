return (function()
  
  local bee = 100
  
  local buzz = bee
  
  local foo = bee
  
  print(bee, buzz, foo, nil)
  return {
    print = print,
    bee = bee,
    buzz = buzz,
    foo = foo,
  }
end)()