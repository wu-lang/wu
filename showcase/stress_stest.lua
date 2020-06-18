return (function()
  function babs(x)
    x = nil
    return x
  end
  
  (function()
    while true do
      repeat
      print("hey")
      break
      print("not hey")
      until true
    end
  end)()
  return {
    babs = babs,
    print = print,
  }
end)()