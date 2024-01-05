## text-layout

`text-layout` provides algorithms for laying out text prior to rendering.

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![Apache licensed][apache-badge]][apache-url]

[crates-badge]: https://img.shields.io/crates/v/text-layout.svg
[crates-url]: https://crates.io/crates/text_layout
[docs-badge]: https://docs.rs/text_layout/badge.svg
[docs-url]: https://docs.rs/text_layout
[apache-badge]: https://img.shields.io/badge/license-Apache--2.0-blue.svg
[apache-url]: LICENSE

## Features

- `no_std` support for constrained environments

## Usage

Laying out a paragraph for display in a terminal:

```rust
fn layout_paragraph<'a, P: ParagraphLayout>(
    paragraph: &'a str,
    layout: &P,
    max_width: usize,
) -> Vec<&'a str> {
    // Process the paragraph into its items.
    let mut items = Vec::new();
    for c in paragraph.chars() {
        items.push(if c.is_whitespace() && items.len() != 0 {
            Item::Glue {
                width: 1.0,
                stretch: 1.0,
                shrink: 0.0,
            }
        } else {
            Item::Box { width: 1.0 }
        });
    }
    items.push(Item::Penalty {
        width: 0.0,
        cost: f32::NEG_INFINITY,
        flagged: true,
    });

    // Calculate the paragraph's breaks.
    let breaks = layout.layout_paragraph(&items, max_width as f32);

    // Render the laid-out paragraph using the break positions.
    let mut cursor = 0;
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, _) in paragraph.chars().enumerate() {
        if i == breaks[cursor].break_at {
            lines.push(&paragraph[start..i]);
            start = i + 1;
            cursor += 1;
        }
    }
    lines.push(&paragraph[start..]);
    lines
}

fn layout_text() -> Result<String, fmt::Error> {
    let text = "  Far out in the uncharted backwaters of the unfashionable end of the western spiral arm of the Galaxy lies a small unregarded yellow sun. Orbiting this at a distance of roughly ninety-two million miles is an utterly insignificant little blue-green planet whose ape-descended life forms are so amazingly primitive that they still think digital watches are a pretty neat idea.";
    let knuth_plass = KnuthPlass::new().with_threshold(f32::INFINITY);
    let lines = layout_paragraph(&text, &knuth_plass, 80);
    let mut result = String::new();
    writeln!(&mut result, "┏{}┓", "━".repeat(80))?;
    for l in lines {
        let pad = 80 - l.chars().count();
        writeln!(&mut result, "┃{}{}┃", l, " ".repeat(pad))?;
    }
    writeln!(&mut result, "┗{}┛", "━".repeat(80))?;
    Ok(result)
}

fn main() -> Result<(), fmt::Error> {
    print!("{}", layout_text()?);
    Ok(())
}
```

This prints:

```console
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃  Far out in the uncharted backwaters of the unfashionable end of the western   ┃
┃spiral arm of the Galaxy lies a small unregarded yellow sun. Orbiting this      ┃
┃at a distance of roughly ninety-two million miles is an utterly insignificant   ┃
┃little blue-green planet whose ape-descended life forms are so amazingly        ┃
┃primitive that they still think digital watches are a pretty neat idea.         ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```
