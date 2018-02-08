## the wu lang
[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

A strongly typed programming language for a happy workflow.

### syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.github.io).

#### taster

```=
fib :: (a: int) int -> match a {
    | 0 -> 0
    | 1 -> 1
    | _ -> fib(a - 1) + fib(a - 2)
}

fibs := list new()

for i, 100 {
    fibs push(fib(i))
}

println(fibs)
```

### bla bla

The language strives to be easy to use on a syntax- as w[](https://)ell as a code structure level. It is a smooth mix of what we've found to be the neatest and lovliest qualities of the languages we're used to; Rust, MoonScript, Elm(*and the function family*), ~~Java~~ *Python* and Nim. This with weight on the making design choices like Data Oriented Design and functional program composition feel somewhat natural to use, coming from any alternative.

### contributors

nilq: https://github.com/nilq
FuzzyLitchi: https://github.com/FuzzyLitchi

### license
MIT