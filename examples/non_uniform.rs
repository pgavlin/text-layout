extern crate text_layout;
use figlet_rs::FIGlet;
use std::fmt::{self, Write};
use text_layout::{Item, KnuthPlass, LineWidths, ParagraphLayout};

fn render_drop_cap(first: char) -> Vec<String> {
    FIGlet::small()
        .unwrap()
        .convert(&first.to_string())
        .unwrap()
        .to_string()
        .lines()
        .filter(|row| !row.trim().is_empty())
        .map(|row| row.trim_end().to_owned())
        .collect()
}

struct DropCapWidths<'a> {
    width: usize,
    rows: &'a [String],
}

impl LineWidths for DropCapWidths<'_> {
    fn line_width(&self, line: usize) -> f32 {
        // The first text line sits beside the second row of the drop cap.
        let inset = self
            .rows
            .get(line + 1)
            .map_or(0, |row| row.chars().count() + 1);
        (self.width - inset) as f32
    }

    fn uniform_at(&self) -> Option<usize> {
        Some(self.rows.len().saturating_sub(1))
    }
}

fn layout_paragraph<'a, 'cap, P: ParagraphLayout<(), (), (), f32, DropCapWidths<'cap>>>(
    paragraph: &'a str,
    layout: &P,
    widths: DropCapWidths<'cap>,
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
    let breaks = layout.layout_paragraph(&items, widths);

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
    let text = "Far out in the uncharted backwaters of the unfashionable end of the western spiral arm of the Galaxy lies a small unregarded yellow sun. Orbiting this at a distance of roughly ninety-two million miles is an utterly insignificant little blue-green planet whose ape-descended life forms are so amazingly primitive that they still think digital watches are a pretty neat idea.";
    let knuth_plass = KnuthPlass::new().with_threshold(f32::INFINITY);

    // Render the first letter separately, reserving space beside it for the text.
    let mut chars = text.chars();
    let first = chars.next().unwrap();
    let rows = render_drop_cap(first);

    let lines = layout_paragraph(
        chars.as_str(),
        &knuth_plass,
        DropCapWidths {
            width: 80,
            rows: &rows,
        },
    );
    let mut result = String::new();
    writeln!(&mut result, "┏{}┓", "━".repeat(80))?;
    for i in 0..(lines.len() + 1).max(rows.len()) {
        let mut line = String::new();
        if let Some(row) = rows.get(i) {
            write!(&mut line, "{} ", row)?;
        }
        if let Some(text) = i.checked_sub(1).and_then(|i| lines.get(i)) {
            line.push_str(text);
        }
        let pad = 80 - line.chars().count();
        writeln!(&mut result, "┃{}{}┃", line, " ".repeat(pad))?;
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
    fn drop_cap_widths() {
        let rows = render_drop_cap('F');
        assert!(rows.iter().all(|row| !row.trim().is_empty()));
        let widths = DropCapWidths {
            width: 80,
            rows: &rows,
        };
        for (i, row) in rows.iter().skip(1).enumerate() {
            assert_eq!(widths.line_width(i), (80 - row.chars().count() - 1) as f32);
        }
        assert_eq!(widths.line_width(rows.len() - 1), 80.0);
        assert_eq!(widths.uniform_at(), Some(rows.len() - 1));
    }

    #[test]
    fn non_uniform() {
        let expected = r#"┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ ___                                                                            ┃
┃| __| ar out in the uncharted backwaters of the unfashionable end of the western┃
┃| _| spiral arm of the Galaxy lies a small unregarded yellow sun. Orbiting      ┃
┃|_| this at a distance of roughly ninety-two million miles is an utterly        ┃
┃insignificant little blue-green planet whose ape-descended life forms are so    ┃
┃amazingly primitive that they still think digital watches are a pretty neat     ┃
┃idea.                                                                           ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
"#;
        let actual = layout_text().unwrap();
        assert_eq!(actual, expected);
    }
}
