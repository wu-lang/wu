<img align="right" width="30%" height="30%" src="https://preview.ibb.co/ePa1eH/wu_dragon.png" alt="wu_dragon">

# Wu

[![Foo](https://user-images.githubusercontent.com/7288322/34429152-141689f8-ecb9-11e7-8003-b5a10a5fcb29.png)](https://discord.gg/qm92sPP)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wu-lang/wu/blob/master/LICENSE)

An expression oriented, strongly typed and sweet language

## Syntax

A full walk-through of the language can be found over at the [wu-lang documentation](https://wu-lang.github.io/wu.html).

### A decent language

Apart from being the best language ever, Wu strives to be a decently efficient, control-focused high-level language for use in game development as well as general purpose development. Its syntax is highly inspired by Rust's strong *explicit* syntax, combined with concepts from Jonathan Blow's Jai language and the sugar of MoonScript and the functional family.

The language is meant and designed to be a solid alternative to MoonScript, and even superior on control and scalability.

### Taster

```swift
-- making a hundred balls


-- an anonymous structure
ball := {
  x: float = 100
  y: float = 100
}

-- infinite arrays
balls: [ball] = []

i := 0
i = while i < 100 {
  balls[i] = ball {
    math random(0, 100)
    math random(0, 100)
  }
  
  i + 1
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

- [evolbug](https://github.com/evolbug)

### License

[MIT License](https://github.com/wu-lang/wu/blob/master/LICENSE)
