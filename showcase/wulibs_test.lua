return (function()
  package.path = package.path .. ';/home/nielsh/.wu/libs/?.lua;/home/nielsh/.wu/libs/?/init.lua'
  local test = require('test')
  local lol = test['lol']
  
  
  return {
    lol = lol,
  }
end)()