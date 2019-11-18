return (function()
  local love = (function()
    local graphics = (function()
      function set_color(r, g, b)
        return setColor((r / 255), (g / 255), (b / 255))
      end
      return {
        setColor = setColor,
        set_color = set_color,
        rectangle = rectangle,
        circle = circle,
        line = line,
        push = push,
        pop = pop,
        translate = translate,
        rotate = rotate,
      }
    end)()
    local keyboard = (function()
      
      return {
        isDown = isDown,
        keypressed = keypressed,
        keyreleased = keyreleased,
      }
    end)()
    function load()
    endfunction update(dt)
    endfunction draw()
    end
    return {
      graphics = graphics,
      keyboard = keyboard,
      load = load,
      update = update,
      draw = draw,
    }
  end)()
  
  local BigFoo = {}
  
  love['load'] = function()
    print("hey")
  end
  return {
    love = love,
    BigFoo = BigFoo,
  }
end)()