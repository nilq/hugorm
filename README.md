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
let tensor = [1, 2, 3, 4]

fun normalize(a):
    let len = 0
    
    for n in a:
      len += a^2
    
    len^0.5

normalize(tensor)
```

## The future

- Gradual typing
- First class things: tensors, graphics stuff
- More speed

## License

MIT
