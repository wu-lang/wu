return (function()
  local babs = function(x)
    x = (x + 1)
    return x
  end
  
  return {
    babs = babs,
  }
end)()