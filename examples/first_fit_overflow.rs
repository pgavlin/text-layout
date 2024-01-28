extern crate text_layout;
use std::fmt::{self, Write};
use text_layout::{FirstFit, Item, ParagraphLayout};

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
                data: (),
            }
        } else {
            Item::Box {
                width: 1.0,
                data: (),
            }
        });
    }
    items.push(Item::Glue {
        width: 0.0,
        stretch: 100000.0,
        shrink: 0.0,
        data: (),
    });
    items.push(Item::Penalty {
        width: 0.0,
        cost: f32::NEG_INFINITY,
        flagged: true,
        data: (),
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
    let text = "FaroutintheunchartedbackwatersoftheunfashionableendofthewesternspiralarmoftheGalaxy lies a small unregarded yellow sun. Orbiting this at a distance of roughly ninety-two million miles is an utterly insignificant little blue-green planet whose ape-descended life forms are so amazingly primitive that they still think digital watches are a pretty neat idea.";
    let first_fit = FirstFit::new()
        .with_threshold(f32::INFINITY)
        .allow_overflow(true);
    let lines = layout_paragraph(&text, &first_fit, 80);
    let mut result = String::new();
    writeln!(&mut result, "┏{}┓", "━".repeat(80))?;
    for l in lines {
        let pad = 80_usize.saturating_sub(l.chars().count());
        writeln!(&mut result, "┃{}{}┃", l, " ".repeat(pad))?;
    }
    writeln!(&mut result, "┗{}┛", "━".repeat(80))?;
    Ok(result)
}

fn main() -> Result<(), fmt::Error> {
    print!("{}", layout_text()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readme() {
        let expected = r#"┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃FaroutintheunchartedbackwatersoftheunfashionableendofthewesternspiralarmoftheGalaxy┃
┃lies a small unregarded yellow sun. Orbiting this at a distance of roughly      ┃
┃ninety-two million miles is an utterly insignificant little blue-green planet   ┃
┃whose ape-descended life forms are so amazingly primitive that they still think ┃
┃digital watches are a pretty neat idea.                                         ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
"#;
        let actual = layout_text().unwrap();
        assert!(actual == expected);
    }
}
