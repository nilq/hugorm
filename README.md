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
