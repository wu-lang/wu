return (function()
  package.path = package.path .. ';/home/nielsh/.wu/libs/?.lua;/home/nielsh/.wu/libs/?/init.lua'
  local test = require('test')
  local lol = test['lol']
  
  
  package.path = package.path .. ';/home/nielsh/.wu/libs/?.lua;/home/nielsh/.wu/libs/?/init.lua'
  local lover = require('lover')
  local graphics = lover['graphics']
  
  
  graphics['setColor'](1, 1, 0)
  local library = require('showcase.library')
  
  
  return {
    lol = lol,
    graphics = graphics,
    library = library,
    love = love,
  }
end)()