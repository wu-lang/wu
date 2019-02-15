return (function()
  local something = (function()
    local set_color = love.graphics.setColor
    local rectangle = love.graphics.rectangle
    
    return {
      set_color = set_color,
      rectangle = rectangle,
    }
  end)()
  
  local draw = love.draw
  
  draw = function()
    something['set_color'](1, 1, 0)
    return something['rectangle']("fill", 100, 100, 200, 150)
  end
  return {
    something = something,
    draw = draw,
  }
end)()