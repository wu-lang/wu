return (function()
  local foo = { 1, 1 }
  
  function bar()
    return foo
  end
  
  function lol()
    return 1, 2, 3
  end
  
  return {
    foo = foo,
    bar = bar,
    lol = lol,
  }
end)()