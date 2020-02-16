# hugorm

```fs
# milestone stuff
let a = 10
let b = a
let c = a + b

fn add(a, b):
    a + b

let d = add(b, c)
```

## Q & A

> Is it fast?

Yes.

> How is it fast?

Hugorm has its own virtual machine with a strict data layout - using *nantagging*. This means that all values are represented as doubles, but used differently through dirty tricks.

> Types?

Sometimes.