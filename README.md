## wu
a neat wannabe low-level programming language

---

### features

#### currently

- transpilation to lua

- function variants

- type safety

- powerful

- being the language of the future

#### in the future

- partial application, currying

- an interpreter

---

### version 0.0.1

bindings
```lua
foo := .1234               -- inferred variable
bar: string = "swordfight" -- explicit variable
baz :: true                -- inferred immutable
boo: float: 3.14159265     -- explicit immutable
```

block-expression
```lua
-- just a local scope
{
 foo := 100
 bar :: foo + 100
 
 print(foo, bar)
}

-- blocks return implicitly returns
-- their last value ..
foo :: {
 baz :: (a float) float -> a^10
 baz(100)
}

bar: bool = {
 return true -- "return" = wrong grrrr
}
```

types
```lua
int float bool string
```

functions
```lua
-- functions also implicitly return
add_5 :: (a: int) int -> a + 5

apply :: (fun: (int) int, a: int) -> fun(a)

ten: int = apply(add_5, 15)

-- or not
sub_5 :: (a: int) int -> return a - 5
sub_0 :: (a: int) int -> {
 return a - 0 -- "return" = sure
}
```

```lua
-- btw. pipe operators(can only one argument(currently))
fifteen := 10 |> add_5
fifteen := add_5 <| 10
```

parameter defaults
```lua
bar :: (a: float, b: float = 100.0) float -> {
  return a + b
}

print(add(100))        -- 200
print(add(100, 200.5)) -- 300.5
```

function-type
`(type*) type*` e.g. `(int, int) int`(taking two ints, returning int)

```lua
sub: (int, int) int = (a: int, b: int) int -> a - b
```

---

### inspiration

- the thing about transpiling to lua, from moonscript

- the weird argument type order, from go

- the lack of inconsistency, not from javascript

- ~~function calls and~~ *the* operators, from haskell/elm etc.

- low-level feel and control, from kai/rust
