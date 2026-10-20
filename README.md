# fenth

Small dependency-free collection of color themes. Every theme has the same set of semantic roles.

Themes and roles are plain data files. Adding one never touches the source code.

[**WARNING**]: Fenth is largely AI-assisted, and though I hardly doubt this will matter for your project, it's still worth knowing about.

```rust
let theme = fenth::get("catppuccin-mocha").unwrap();

theme["accent.primary"];
theme.get("accent.primary");
theme.hex("accent.primary");

for (role, color) in theme.iter() {
    println!("{role} {}", color.to_hex());
}

for name in fenth::names() {
    println!("{name}");
}
```

`theme[...]` panics on an unknown role name, `get` and `hex` return `None`.

## Adding a theme

Drop a file into `themes/`. It is picked up on the next build, no other edits needed.

```
name = my-theme
variant = dark

background = #1e1e2e
surface = #313244
overlay = #45475a
selection = #585b70
border = #6c7086
foreground = #cdd6f4
foreground.muted = #a6adc8
accent.primary = #cba6f7
accent.secondary = #b4befe
red = #f38ba8
orange = #fab387
yellow = #f9e2af
green = #a6e3a1
cyan = #94e2d5
blue = #89b4fa
purple = #cba6f7
pink = #f5c2e7
```

Those 17 roles are the required ones. Everything else (hover states, tinted backgrounds, syntax, terminal colors) is derived from them, and any of it can be overridden by adding a line.

A value can be:

- a hex color: `#rrggbb`, `#rgb` or `#rrggbbaa`
- another role: `warning = orange`
- a mix of two colors: `error.background = mix(background, red, 20%)`

Blank lines are ignored, and so are lines starting with `#`. The build fails with a message naming the file and line if a theme has a typo in a role name, a bad color, a missing required role, or a duplicate name.

## Adding a role

Add one line to `roles.list`:

```
sidebar.badge = accent.primary
```

A role with a default (the part after `=`) needs no changes in any theme. Themes can override it whenever they like. A role without a default becomes required, and the build lists every theme that is missing it.

## Loading themes at runtime

```rust
let theme = fenth::Theme::from_file("my-theme.theme")?;
let themes = fenth::load_dir("themes")?;
let theme = fenth::Theme::parse(&source)?;
```

Runtime themes use the same format and are checked against the same `roles.list` that was compiled in.

## Roles

The full list, with defaults, is in `roles.list`. Groups:

| Group | Roles |
| --- | --- |
| surfaces | `background` `background.alt` `surface` `surface.alt` `overlay` `highlight` `active` `selection` `selection.inactive` `scrollbar` `scrollbar.hover` |
| borders | `border` `border.muted` `border.focus` `divider` |
| text | `foreground` `foreground.muted` `foreground.subtle` `foreground.disabled` `foreground.inverse` `placeholder` `link` `link.hover` |
| accents | `accent.primary` `accent.primary.hover` `accent.primary.active` `accent.secondary` `accent.secondary.hover` `accent.tertiary` |
| hues | `red` `orange` `yellow` `green` `cyan` `blue` `purple` `pink` |
| status | `error` `warning` `success` `info`, each with a `.background` tint |
| editor | `cursor` `line.number` `line.number.active` `line.highlight` `search.match` `search.match.active` `bracket.match` `indent.guide` `whitespace` |
| diff | `diff.added` `diff.removed` `diff.modified`, each with a `.background` tint |
| syntax | `syntax.` + `comment` `keyword` `string` `number` `function` `type` `variable` `constant` `operator` `punctuation` `property` `tag` `attribute` `escape` `macro` |
| terminal | `terminal.` + `black` `red` `green` `yellow` `blue` `magenta` `cyan` `white`, and the same eight under `terminal.bright.` |

## Themes

`catppuccin-latte`, `catppuccin-mocha`, `dracula`, `gruvbox-dark`, `monokai`, `nord`, `one-dark`, `rose-pine`, `solarized-dark`, `solarized-light`, `tokyo-night`

If a palette has no real equivalent for a role (Nord has no pink, Dracula has no blue), it reuses the closest one.

## Color

```rust
use fenth::Color;

let c: Color = "#cba6f7".parse().unwrap();

c.to_hex();
c.to_rgb();
c.to_rgb_f32();
c.to_css_rgb();
c.to_hsl();
c.mix(Color::WHITE, 0.2);
Color::from_hsl(267.0, 0.92, 0.82);
Color::from_hex(0x1e1e2e);
```

## Preview in the terminal

```bash
cargo run --example preview
cargo run --example preview nord
cargo run --example preview path/to/my-theme.theme
```

The last form re-reads the file each time, so you can edit a theme and preview it without rebuilding the crate.

## Ratatui demo

A small app with a switchable theme list, a palette, sample text, a syntax-highlighted snippet and status messages, all colored through fenth.

```bash
cargo run --example ratatui_demo
cargo run --example ratatui_demo path/to/my-theme.theme
```

`↑`/`↓` or `j`/`k` switch themes, `g`/`G` jump to the first or last, `q` quits. Extra `.theme` files passed as arguments are added to the list. It needs a terminal of at least 80x24.

ratatui is only a dev-dependency, so the crate itself still has no dependencies.
