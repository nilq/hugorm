<img align="right" width="30%" height="30%" src="https://i.ibb.co/jT8XDmz/hugorm.png" alt="hugorm">

# Hugorm

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/nilq/hugorm/blob/master/LICENSE)


## What is Hugorm?

It is a dynamically typed programming language. Hugorm strives to be *better* version of the other snake language, with a heightened focus on \<good things here\>. The syntax is designed to be easy to read and easy to write, and runs on a constantly improving bytecode virtual machine.

### Selling points

> "Sssssss ..." - hugorm

- [x] Decently fast
- [x] Easy-to-learn syntax
- [x] Made in Rust
- [x] The REPL has colors
- [ ] ~~Absolutely massive community~~
- [x] Surprising bugs, maybe

## Examples

Everyone loves examples. Below are some rather useless example programs, showcasing the raw syntax of Hugorm.

### Functions
> Literally fun.

```fs
fun hello(name):
    print("hello " ++ name)

let name = prompt()
hello(name)
```

### Loops

```fs
fun banana(n):
  let i = 0
  while i < n:
    if i == 0:
      print("hey")
    else:
      print("hey again")
  
    i = i + 1
    
banana(1000)
```

#### Exotic loops

```lua
repeat 100:
  print("just like 100 times")
```

```rust
loop:
  print("while true")
```

### Data

The code below will print `200`.

```fs
let player = {
  x: 100
  y: 100
}

let foo = {
    x: player.x + 100
}

player = foo

print(player.x)
```

## The future

### Interfaces

An interface will work like a set of pre-made functions that can be bound to new objects. Kinda like the way you implement traits on a struct in Rust.

```fs
interface Moving:
  fun move(self, x, y):
    self.x += x
    self.y += y

let snake = {} with Moving
```

## TO-DO

- [ ] Concat is sometimes wonky
- [ ] Stack overflowing when doing extreme loops

## License

MIT
