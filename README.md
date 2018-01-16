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

### syntax

```
foo := .1234               -- inferred variable
bar: string = "swordfight" -- explicit variable
baz :: true                -- inferred constant
```

```
add :: (a int, b int) int -> a + b
fac :: (a int) int -> match {
  | 1 -> 1
  | _ -> fac(n - 1) * n
}
```

```
sub: (int, int) int = (a int, b int) int -> a - b
```

---

### inspiration

- the thing about transpiling to lua, from moonscript

- the weird argument type order, from go

- the lack of inconsistency, not from javascript

- ~~function calls and~~ operators, from haskell/elm etc.

- low-level feel and control, from kai/rust
