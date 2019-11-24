return (function()
  
  local i = 0
  
  (function()
    for __iterator_0 = 1, 10 do
      
      local __brk_0 = false
      repeat
      if (i >= 5) then
      do
        __brk_0 = true break
      end
      end
      print("ten times!")
      i = (i + 1)
      until true
      if __brk_0 then break end
    end
  end)()
  local list = {
    [1] = 1,
    [2] = 2,
    [3] = 3,
    [4] = 3
  }
  
  
  (function()
    for x, y in ipairs(list) do  
      local __brk_0 = false
      repeat
      print(x, y)
      until true
      if __brk_0 then break end
    end
  end)()
  return {
    print = print,
    i = i,
    list = list,
    ipairs = ipairs,
  }
end)()