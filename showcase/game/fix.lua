return (function()
  local aaaa = function()
    if true then
        return "10"
    else
        return "10"
    end
  end

  return {
    aaaa = aaaa,
  }
end)()