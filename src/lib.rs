#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "fixed", feature = "libm")))]
compile_error! { "Either the std, fixed, or libm feature must be enabled" }

extern crate alloc;
use alloc::vec::Vec;

mod first_fit;
pub use first_fit::*;

mod knuth_plass;
pub use knuth_plass::*;

mod math;
pub use math::{Fixed, Num};

/// A single item in a paragraph.
#[derive(Debug)]
pub enum Item<N = f32> {
    /// An unbreakable box containing paragraph content. Typically represents a glyph or sequence
    /// of glyphs. Lines may not be broken at boxes.
    Box {
        /// The width of the box.
        width: N,
    },
    /// Whitespace that separates boxes. Lines may be broken at glue items.
    Glue {
        /// The normal width of the whitespace.
        width: N,
        /// The stretch parameter. If this item needs to be stretched in order to lay out a line,
        /// the stretch amount will be proportional to this value.
        stretch: N,
        /// The shrink parameter. If this item needs to be shrunk in order to lay out a line, the
        /// shrink amount will be proportional to this value.
        shrink: N,
    },
    /// A penalty item. Represents a possible breakpoint with a particular aesthetic cost that
    /// indicates the desirability or undesirability of such a breakpoint.
    Penalty {
        /// The width of the penalty item.
        width: N,
        /// The aesthetic cost of the penalty item. A high cost is a relatively undesirable
        /// breakpoint, while a low cost indicates a relatively desirable breakpoint.
        cost: N,
        /// Whether or not this is a flagged penalty item. Some algorithms will attempt to avoid
        /// having multiple consecutive breaks at flagged penalty items.
        flagged: bool,
    },
}

impl<N: Num> Item<N> {
    fn penalty_cost(&self) -> N {
        match self {
            Item::Penalty { cost, .. } => *cost,
            _ => N::from(0i16),
        }
    }

    fn penalty_flag(&self) -> N {
        match self {
            Item::Penalty { flagged, .. } => {
                if *flagged {
                    N::from(1i16)
                } else {
                    N::from(0i16)
                }
            }
            _ => N::from(0i16),
        }
    }

    fn is_mandatory_break(&self) -> bool {
        match self {
            Item::Penalty { cost, .. } => *cost == N::NEG_INFINITY,
            _ => false,
        }
    }

    /// Returns the width, stretch, and shrink of the node at b and indicates whether or not b is a
    /// legal break.
    fn is_legal_breakpoint(&self, pred: Option<&Item<N>>) -> (N, N, N, bool) {
        match self {
            Item::Box { width } => (*width, N::from(0), N::from(0), false),
            Item::Glue {
                width,
                stretch,
                shrink,
            } => (
                *width,
                *stretch,
                *shrink,
                matches!(pred, Some(Item::Box { .. })),
            ),
            Item::Penalty { width, cost, .. } => {
                (*width, N::from(0), N::from(0), *cost != N::INFINITY)
            }
        }
    }

    /// Calculates the adjustment ratio for a break at the given item. Width, stretch, and shrink
    /// are for the line that ends at the break.
    fn adjustment_ratio(&self, width: N, stretch: N, shrink: N, line_width: N) -> N {
        let penalty_width = if let Item::Penalty { width, .. } = self {
            *width
        } else {
            N::from(0)
        };
        let width = width + penalty_width;
        if width < line_width {
            if stretch > N::from(0) {
                (line_width - width) / stretch
            } else {
                N::INFINITY
            }
        } else if width > line_width {
            if shrink > N::from(0) {
                (line_width - width) / shrink
            } else {
                N::NEG_INFINITY
            }
        } else {
            N::from(0)
        }
    }
}

/// A single line of text as represented by its break point and adjustment ratio.
#[derive(Debug, Default, Clone, Copy)]
pub struct Line<N: Num = f32> {
    /// The index of the item at which to break this line.
    pub break_at: usize,
    /// The adjustment ratio that should be applied to glue when rendering this line. If the
    /// adjustment ratio is negative, glue should be adjusted by its shrink parameter. If the
    /// adjustment ratio is positive, glue should be adjusted by its stretch parameter. In general,
    pub adjustment_ratio: N,
}

impl<N: Num> Line<N> {
    /// Returns the width of a glue item with the given width, stretch, and shrink once the
    /// adjustment ratio is taken into account.
    pub fn glue_width(&self, width: N, stretch: N, shrink: N) -> N {
        if self.adjustment_ratio < N::from(0i16) {
            width + shrink * self.adjustment_ratio
        } else if self.adjustment_ratio > N::from(0i16) {
            width + stretch * self.adjustment_ratio
        } else {
            width
        }
    }
}

/// Represents a paragraph layout algorithm
pub trait ParagraphLayout<N: Num = f32> {
    /// Lays out a paragraph with the given line width that consists of as list of items and
    /// returns the laid-out lines.
    fn layout_paragraph(&self, items: &[Item<N>], line_width: N) -> Vec<Line<N>>;
}
