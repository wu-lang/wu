return (function()
  local a = 10
  
  
  local b = (function()
    local __switch_tmp_38 = a
    if (0 == __switch_tmp_38) then
      local a = 10
      print("hey")
      return nil
    elseif (1 == __switch_tmp_38) then
      print("hello hoy yuo")
    elseif (10 == __switch_tmp_38) then
      print("it was 10??? all along")
    end
  end)()
  
  return {
    a = a,
    print = print,
    b = b,
  }
end)()