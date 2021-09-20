# vim-slides.rs

Generate slides from a single markdown file to present in vim.

Most likely you don't want to use this software as I have no idea what I am doing in rust.

## Usage

You pass a path to the source file

```sh
cargo run ./source.md ./slides/destination
```

Where the `./source.md` has following content

```md
# Title slide

text

## Secondary slide

more text

## Outro slide

references
```

as the result you get

`./slides/001.md`

```md
# Title slide

text
```

`./slides/002.md`

```md
## Secondary slide

more text
```

`./slides/003.md`

```md
## Outro slide

references
```

### Watch mode

Alternately you can supply `--watch` flag to monitor for file changes and rebuild slides.

```sh
vim-slides --watch ./source.md ./slides/destination
```
