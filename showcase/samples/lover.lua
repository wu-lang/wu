return (function()
  local love = 
  
  love['conf'] = function(t)
    t['window']['width'] = 800
  end
  
  return {
    love = love,
  }
end)()