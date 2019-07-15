# xdg-open

Basically a drop-in replacement to xdg-open usage-wise.  It uses a different
config though and won't try to guess programs to use.


## Config

xdg-open looks for a config file at the path contained in `MIME_MAP_FILE` first,
and then looks for a file called `mime_map.toml` in the current directory.  If
neither one exists or is invalid toml, there is an error.

The config file consists of mime type headers followed by subtype entrys.

```toml
[application]
pdf = "zathura --fork"

[text]
html = "luakit -U"
x-rust = "nvim"

[audio]
wav = "mpv"
webm = "mpv"

[video]
mpeg = "mpv"
webm = "mpv"

[image]
jpeg = "feh -."
png = "feh -."
```

MIME types are guessed by [mime_guess](https://crates.io/crates/mime_guess).  If
you don't know the mime type of a file try xdg-open-ing it.

```sh
$ xdg-open README.md
I don't know how to open 'README.md' (text/x-markdown)
```

Then add the relevant entry to your config file.

```toml
[text]
x-markdown = "nvim"
```

## Notes

### URI handling

Prior to mime type guessing, a simple regex is applied to the `path` argument:
`https?://.*`.  If it matches, the mime type is set to `text/html`.  Additional
protocols may be supported in the future.

### Default MIME

If no mime type is detected, xdg-open falls back to `application/octet-stream`.
