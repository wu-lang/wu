return (function()
  
  local bee = 110
  
  local buzz = bee; assert(bee ~= nil, "can't unwrap 'nil'")
  
  local foo = bee
  
  print(bee, buzz, foo, nil)
  function foo()
    return {
      [1] = 1,
      [2] = 2,
      [3] = 3,
      [4] = 4
    }
  end
  return {
    print = print,
    bee = bee,
    buzz = buzz,
    foo = foo,
    foo = foo,
  }
end)()