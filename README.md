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

<!-- speaker note 1 -->

## Secondary slide

more text

<!-- speaker note 2 -->

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

### Speaker notes

To preserve speaker notes from the slides(written in html comments) pass `--notes` flag with a path to where the notes should be saved

```sh
vim-slides --notes ./notes.md ./source.md ./slides/destination
```

Notes in `./notes.md`

```md
# Speaker notes

# Title slide

speaker note 1

## Secondary slide

speaker note 2

## Outro slide

empty
```
