return (function()
  local love_wrapper = require('showcase/game/love_wrapper')
  
  
  
  
  love['load'] = function()
    return print("safkj")
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