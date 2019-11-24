return (function()
  local love_wrapper = require('showcase/game/love_wrapper')
  
  
  
  
  love['load'] = function()
    local x = (function()
    if true then
      return "hey"
    else
        return "eh"
    end
    end)()
    return print("safkj", x)
  end
  love['draw'] = function()
    love['graphics']['setColor'](1, 0, 1, 1)
    return love['graphics']['rectangle']("fill", 100, 100, 100, 100)
  end
  return {
    love_wrapper = love_wrapper,
    love = love,
    print = print,
  }
end)()