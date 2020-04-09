return (function()
  local File = {}
  
  
  local file = setmetatable({
  }, {__index=File})
  
  file['close']()
  return {
    File = File,
    file = file,
  }
end)()