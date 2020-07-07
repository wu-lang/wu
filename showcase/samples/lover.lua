return (function()
  love['conf'] = function(t)
    t['window']['width'] = 800
  end
  
  return {
    love = love,
  }
end)()