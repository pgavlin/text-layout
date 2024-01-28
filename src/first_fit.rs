extern crate alloc;
use alloc::vec::Vec;

use crate::math::Num;
use crate::{Item, Line, ParagraphLayout};

/// Runs the first-fit line-breaking algorithm to calculate the break points for a paragraph.
pub struct FirstFit<N> {
    threshold: N,
    allow_overflow: bool,
}

impl<N: Num> FirstFit<N> {
    /// Creates a new FirstFit layout with default parameter values.
    pub fn new() -> Self {
        FirstFit {
            threshold: N::from(1),
            allow_overflow: false,
        }
    }

    /// Sets the adjustment ratio threshold. Lines will not be allowed to break at a given point if
    /// doing so would cause the line's adjustment ratio to exceed this value. Defaults to 1.
    pub fn with_threshold(mut self, threshold: N) -> Self {
        self.threshold = threshold;
        self
    }

    /// Configures the layout to allow lines that exceed the maximum line with if the layout would
    /// fail otherwise.
    pub fn allow_overflow(mut self, allow_overflow: bool) -> Self {
        self.allow_overflow = allow_overflow;
        self
    }
}

impl<N: Num> Default for FirstFit<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Box, Glue, Penalty, N: Num> ParagraphLayout<Box, Glue, Penalty, N> for FirstFit<N> {
    fn layout_paragraph(
        &self,
        items: &[Item<Box, Glue, Penalty, N>],
        line_width: N,
    ) -> Vec<Line<N>> {
        let l = FirstFitLayout {
            line_width,
            threshold: self.threshold,
            allow_overflow: self.allow_overflow,
            width: N::from(0),
            stretch: N::from(0),
            shrink: N::from(0),
            lines: Vec::new(),
        };
        l.layout_paragraph(items)
    }
}

struct Break<N> {
    width: N,
    stretch: N,
    shrink: N,
    adjustment_ratio: N,
    is_mandatory: bool,
    at: usize,
}

struct FirstFitLayout<N: Num> {
    line_width: N,

    threshold: N,
    allow_overflow: bool,

    width: N,
    stretch: N,
    shrink: N,

    lines: Vec<Line<N>>,
}

impl<N: Num> FirstFitLayout<N> {
    fn break_at(&mut self, b: Break<N>) {
        self.lines.push(Line {
            break_at: b.at,
            adjustment_ratio: b.adjustment_ratio,
        });

        self.width -= b.width;
        self.stretch -= b.stretch;
        self.shrink -= b.shrink;
    }

    fn layout_paragraph<Box, Glue, Penalty>(
        mut self,
        items: &[Item<Box, Glue, Penalty, N>],
    ) -> Vec<Line<N>> {
        let mut last_breakpoint: Option<Break<N>> = None;
        for (b, item) in items.iter().enumerate() {
            let (width, stretch, shrink, is_legal) =
                item.is_legal_breakpoint((b != 0).then(|| &items[b - 1]));
            if is_legal {
                let adjustment_ratio =
                    item.adjustment_ratio(self.width, self.stretch, self.shrink, self.line_width);
                if let Some(b) = last_breakpoint {
                    if adjustment_ratio < N::from(-1)
                        || adjustment_ratio > self.threshold
                        || b.is_mandatory
                    {
                        self.break_at(b);
                    }
                }

                let adjustment_ratio =
                    item.adjustment_ratio(self.width, self.stretch, self.shrink, self.line_width);

                let adjustment_ratio = if adjustment_ratio < N::from(-1) {
                    if !self.allow_overflow {
                        return Vec::new();
                    }
                    N::from(0)
                } else {
                    adjustment_ratio
                };
                if adjustment_ratio > self.threshold {
                    return Vec::new();
                }

                last_breakpoint = Some(Break {
                    width: self.width,
                    stretch: self.stretch,
                    shrink: self.shrink,
                    adjustment_ratio,
                    is_mandatory: item.is_mandatory_break(),
                    at: b,
                });
            }

            self.width += width;
            self.stretch += stretch;
            self.shrink += shrink;
        }
        if let Some(b) = last_breakpoint {
            self.break_at(b);
        }

        self.lines
    }
}
