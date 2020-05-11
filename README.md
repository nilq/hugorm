<div align="center">
  <img width="30%" height="30%" src="https://i.ibb.co/gStcJrc/hugorm.png" alt="hugorm" border="0"/>
</div>
<h1 align="center">Hugorm</h1>

## What is Hugorm?

Hugorm is a dynamic scripting language designed for mission-critical development, providing a solid stack choice for cool things like:

- small-scale data science projects
- game jam projects
- automatic math assignment tasks
- ... and everything else

### Showcase

Currently the language is in the early stages, so here's a generic, useless example:

```fs
fun a(b):
    fun a'(c):
        return c

    return a'(b)

let foo = {
    bob: 10 * 10
    ild: 100
}

print(a(foo).bob + foo.ild)
```

And a more exotic example:

```fs
interface Move:
  fun move(self, dx, dy):
    self.x += dx
    self.y += dy

let snake = {} with Move
snake\move(10, 10) // Calling method with `self` being snake
```

Compile-time programming:

```kotlin
const foo = 100 + 100

const fun bar(b):
  if b:
    return "ok hello"
  else:
    return "ok hello but false"

const baz = bar(true)
```


## The future

- Gradual typing
- First class things: tensors, graphics stuff
- More speed

## Notes

- The symbol table should store IR bindings rather than `(usize, usize)`

## License

MIT
