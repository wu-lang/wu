return (
function()
  local ___file = setmetatable({}, {__index=_ENV})

  foo = function(a)
    return "hey"
  end
  
  while true do
    do
      b = foo(10)
    end
  end
  
  a = (function()
    while (foo(0) == "hey") do
        foo(0)
        return nil
    end
  end)()
  
  

  _ENV = getmetatable(___file).__index
  return ___file
end)()