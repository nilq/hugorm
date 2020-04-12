<img align="right" width="30%" height="30%" src="https://i.ibb.co/gStcJrc/hugorm.png" alt="hugorm" border="0"/>

# Hugorm 🐍😎

```fs
# milestone stuff
let a = 10
let b = a
let c = a + b

fn add(a, b):
    return a + b

let d = add(b, c)
```

## Todo

- [x] Fast bytecode VM
- [x] Lazy compiler
- [ ] Nan-tagging and RTTI
- [ ] REPL
- [ ] Rust functions
- [ ] Heap; arrays and tables
- [ ] Flow-control
- [ ] GC or lifetimes
- [ ] Profit

## Q & A

> Is it fast?

Yes.

> How is it fast?

Hugorm has its own virtual machine with a strict data layout. This means that all values are represented as doubles, but used differently through dirty tricks.

> Types?

Sometimes.

## Roadmap

Basics [x]
```fs
let foo = 100
let bar = 200

fn add(a, b):
    return a + b

let c = add(foo, bar)
```

Tables [ ]
```fs
let snake = {
    name: "hugorm",
    teeth: "yes"
}
```

Switch [ ]
```fs
fn sssssss(a):
    switch a:
        | "yes" => print("it's yes")
        | "no" => print("it's no")
        | _ => print("it's something else ??")
```

Tuples [ ]
```fs
fn lol():
    return (1, 2)
  
let (a, b) = lol()
```

Bad async [ ]
```fs
async fn add(a, b):
    return a + b
    
# adding in parallel
add(1, 2)
# nice
```

Decorators? [ ]
```fs
@entry
fn main(args):
    print("args are cool, this is the entry")
```
