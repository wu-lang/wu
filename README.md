# Wu

[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

A good purpose, high-control, high-level language.

## Syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.github.io/wu.html).

### A decent language

Apart from being the best language ever, Wu strives to be a decently fast, control-focused high-level language for use in game development as well as general purpose development. Its syntax is highly inspired by Rust's strong *explicit* syntax, combined with concepts from Jonathan Blow's Jai language and the sugar of MoonScript and the functional family.

The language provides and is made to be an alternative to Python and MoonScript/Lua for better scalability and less gross runtime errors.

### Taster

```
fac :: (n: i32) i32 -> match n {
  | 0 -> 0
  | 1 -> 1
  | _ -> fac(n - 1) * n
}

last := 0

loop {
  last = fac(last + 1)

  print("here we go: " ++ last)
}
```

## Building

Currently the Wu compiler relies on the Rust's experimental 128 bit integer types and thus a nightly version of the Rust compiler is required to build the project.

Installation of Nightly Rust can be done as shown in the following.

```
curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly
```

[Further Nightly information](https://doc.rust-lang.org/1.13.0/book/nightly-rust.html)

## Disclaimer

Wu is built by a minimal team of people, all of which are basically kids working on the compiler when bored in class. The whole thing is currently in its very early stages, but is propably fine, go use it in production.

## Contributers

- [nilq](https://github.com/nilq)

- [fuzzylitchi](https://github.com/fuzzylitchi)

- [evolbug](https://githubc.om/evolbug)

### License

[MIT License](https://github.com/wu-lang/wu/blob/master/LICENSE)
