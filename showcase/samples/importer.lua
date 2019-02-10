return (function()
  local traits = require('showcase/samples/traits')
  foo = traits['foo']
  
  
  local lover = require('showcase/samples/lover')
  BigFoo = lover['BigFoo']
  love = lover['love']
  
  
  return {
    foo = foo,
    BigFoo = BigFoo,
    love = love,
  }
end)()