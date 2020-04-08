return (function()
  local foo
  local bar
  foo, bar = 100, 100, 1
  local a, b = 10, 10
  
  local y = b
  
  function waps()
    return 1, 2, 3
  end
  
  local foo, bar, b = (function()
    return 1
  end)(),   return 2
  , 2
  
  return {
    foo = foo,
    bar = bar,
    y = y,
    waps = waps,
  }
end)()