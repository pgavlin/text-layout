extern crate fixed;
extern crate text_layout;

use fixed::types::I16F16;
use std::fmt::{self, Write};
use text_layout::{Fixed, Item, KnuthPlass, ParagraphLayout};

type F = Fixed<I16F16>;

fn layout_paragraph<'a, P: ParagraphLayout<F>>(
    paragraph: &'a str,
    layout: &P,
    max_width: F,
) -> Vec<&'a str> {
    // Process the paragraph into its items.
    let mut items = Vec::new();
    for c in paragraph.chars() {
        items.push(if c.is_whitespace() && items.len() != 0 {
            Item::Glue {
                width: F::from_num(1),
                stretch: F::from_num(1),
                shrink: F::from_num(0),
            }
        } else {
            Item::Box {
                width: F::from_num(1),
            }
        });
    }
    items.push(Item::Glue {
        width: F::from_num(0),
        stretch: F::MAX,
        shrink: F::from_num(0),
    });
    items.push(Item::Penalty {
        width: F::from_num(0),
        cost: F::MIN,
        flagged: true,
    });

    // Calculate the paragraph's breaks.
    let breaks = layout.layout_paragraph(&items, max_width);

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
    let knuth_plass = KnuthPlass::new().with_threshold(F::MAX);
    let lines = layout_paragraph(&text, &knuth_plass, F::from_num(80));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed() {
        let expected = r#"┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃  Far out in the uncharted backwaters of the unfashionable end of the western   ┃
┃spiral arm of the Galaxy lies a small unregarded yellow sun. Orbiting this at a ┃
┃distance of roughly ninety-two million miles is an utterly insignificant little ┃
┃blue-green planet whose ape-descended life forms are so amazingly primitive that┃
┃they still think digital watches are a pretty neat idea.                        ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
"#;
        let actual = layout_text().unwrap();
        assert!(actual == expected);
    }
}
