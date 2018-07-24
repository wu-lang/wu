return (function()
  
  person.new = function(name)
    return {
      name = name,
    }
  end
  
  local bob = person['new']("bob")
  
  return {
    person = person,
    bob = bob,
  }
end)()