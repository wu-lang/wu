## wu
[!discord](https://seeklogo.com/images/D/discord-logo-134E148657-seeklogo.com.png)
a strongly typed language that transpiles to lua

---

### usage

```
wu's transpiler

usage:
    wu <file>
    wu <folder>
```

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

### the syntax

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
```

if and match
```lua
number := math random(0, 100)

if number % 2 == 0 {
  print("okok")
}

-- they're both valid as expressions
hmm := match number % 2 {
 | 0 -> "idc"
 | 1 -> "sure"
}

print(hmm)
```

while
```lua
i := 0

while i < 1000 {
 print("hey")
 i += 1
}
```

```lua
fib :: (a: int) int -> match a {
 | 0 -> 0
 | 1 -> 1
 | a -> fib(a - 1) + fib(a - 2)
}
```

types
```lua
int float bool string
```

arrays
```lua
foo :: [1, 2, 3,]
bar: [string] = ["hey", "grr"]
baz := [false, true, false, false]

hmm := foo[0] -- arrays begin at 0
```

functions
```lua
-- functions also implicitly return
add_5 :: (a: int) int -> a + 5

apply :: (fun: (int) int, a: int) int -> fun(a)

ten: int = apply(add_5, 15)

-- or not
sub_5 :: (a: int) int -> return a - 5
sub_0 :: (a: int) int -> {
 return a - 0 -- "return" = sure
}
```

```lua
-- btw. pipe operators(can only take one argument(currently))
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

structs
```lua
struct point {
 x: float
 y: float
}

position: point = point {
 x: 100
 y: 200
}
```

member indexing
```lua
struct frog {
 position:    point
 jump_height: float
}

bob := frog {
 position:    point { x: 100, y: 100 }
 jump_height: 100000
}

-- very innovative, doesn't have `.`
bob position x = -100
bob jump_height -= 10
```

compound assignments
```lua
a: float = 0
a += 10
a -= 10
a %= 2
a ^= 10
a *= 2
a /= 0.5

s: string = "hello, "
s ++= "world"
```

modules
```lua
module animal {
 struct colibri {
  speed: float
  name: string
 }
 
 struct mouse {
  weight: int
 }
}

expose animal (mouse)

violetear :: animal colibri {
 speed: 1000
 name: "bob"
}

jerry :: mouse {
 weight: 3
}
```

module - other file
```lua
-- given another file named 'hello' exists
-- will put content of hello into a module named 'hello'
module hello
expose hello (*) -- exposes everything
```

---

### inspiration

- the thing about transpiling to lua, from moonscript

- the weird argument type order, from go

- the lack of inconsistency, not from javascript

- ~~function calls and~~ *the* operators, from haskell/elm etc.

- low-level feel and control, from kai/rust
