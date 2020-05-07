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

Currently the language is in the early stages, so here's a generic example:

```fs
fun foo(x):
  return x^3 + 2 * x^2 - 10

fun foo'(x):
  return 3 * x^2 + 4 * x
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

## The future

- Gradual typing
- First class things: tensors, graphics stuff
- More speed

## License

MIT
