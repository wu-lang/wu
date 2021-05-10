return (function()
  local i = 0
  
  for __iterator_0 = 1, 10 do
    
    local __brk_0 = false
    repeat
    if (i >= 5) then
      __brk_0 = true break
    end
    
    print("ten times!")
    i = (i + 1)
    until true
    if __brk_0 then break end
  end
  
  local list = {
    [1] = 1,
    [2] = 2,
    [3] = 3,
    [4] = 3
  }
  
  for x, y in ipairs(list) do  
    local __brk_0 = false
    repeat
    print(x, y)
    until true
    if __brk_0 then break end
  end
  
  function range(a, b)
    local i = 0
    function funky(_, last)
      if ((last + 1) <= b) then
        return (last + 1)
      end
    end
    return funky, nil, (a - 1)
  end
  
  for x in range(0, 100) do  
    local __brk_0 = false
    repeat
    print("going to 100 !!", x)
    until true
    if __brk_0 then break end
  end
  
  return {
    i = i,
    list = list,
    range = range,
  }
end)()