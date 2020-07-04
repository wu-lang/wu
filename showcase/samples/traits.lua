return (function()
  local foo = (function()
    
    return {
      Moving = Moving,
    }
  end)()
  
  local Player = {}
  
  Player['move'] = function(self, dx, dy)
    self['x'] = (self['x'] + dx)
    self['y'] = (self['y'] + dy)
  end
  
  
  
  function here_we_go(hmm)
    hmm['move'](hmm, 10, 10)
    return print((hmm)['x'])
  end
  
  here_we_go(setmetatable({
    x = 100,
    y = 100,
  }, {__index=Player}))
  return {
    foo = foo,
    Player = Player,
    here_we_go = here_we_go,
  }
end)()