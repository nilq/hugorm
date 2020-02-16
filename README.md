<img src="https://i.ibb.co/vsRQftF/received-622991528467925.png" border="0" align="center" />
<h1 align="center">Hugorm</h1>

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
- [ ] Separate function stack
- [ ] Run-time type information
- [ ] Heap; arrays and tables
- [ ] Flow-control
- [ ] GC or lifetimes
- [ ] Rust functions
- [ ] Profit

## Q & A

> Is it fast?

Yes.

> How is it fast?

Hugorm has its own virtual machine with a strict data layout. This means that all values are represented as doubles, but used differently through dirty tricks.

> Types?

Sometimes.
