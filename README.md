## the wu lang
[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

A strongly typed programming language.

### syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.github.io/wu.html).

#### taster

```=
fib :: (a: int) int -> match a {
    | 0 -> 0
    | 1 -> 1
    | _ -> fib(a - 1) + fib(a - 2)
}

fibs := List new()

for i, 100 {
    fibs push(fib(i))
}

print(fibs)
```

### language of the future



### contributors

- nilq: https://github.com/nilq
- FuzzyLitchi: https://github.com/FuzzyLitchi

### license
MIT