return (
function()
  local ___functions = setmetatable({}, {__index=_ENV})  fib = function(a)
    if (a < 3) then
      return a
      
    end
    return (fib((a - 1)) + fib((a - 2)))
  end
  fac = function(a)
    if (a < 3) then
      return "Hey"
      
    end
    return (fac((a - 1)) * a)
  end
  foo = function(a, b)
    return "hey this actually works"
  end
  high = function(a, value)
    return a(value)
  end
  high(fib, 10)
  return ___functions
end)()