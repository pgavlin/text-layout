#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "libm")))]
compile_error! { "Either the std or libm feature must be enabled" }

extern crate alloc;
use alloc::vec::Vec;

mod knuth_plass;
pub use knuth_plass::*;

mod math;

/// A single item in a paragraph.
#[derive(Debug)]
pub enum Item {
    /// An unbreakable box containing paragraph content. Typically represents a glyph or sequence
    /// of glyphs. Lines may not be broken at boxes.
    Box {
        /// The width of the box.
        width: f32,
    },
    /// Whitespace that separates boxes. Lines may be broken at glue items.
    Glue {
        /// The normal width of the whitespace.
        width: f32,
        /// The stretch parameter. If this item needs to be stretched in order to lay out a line,
        /// the stretch amount will be proportional to this value.
        stretch: f32,
        /// The shrink parameter. If this item needs to be shrunk in order to lay out a line, the
        /// shrink amount will be proportional to this value.
        shrink: f32,
    },
    /// A penalty item. Represents a possible breakpoint with a particular aesthetic cost that
    /// indicates the desirability or undesirability of such a breakpoint.
    Penalty {
        /// The width of the penalty item.
        width: f32,
        /// The aesthetic cost of the penalty item. A high cost is a relatively undesirable
        /// breakpoint, while a low cost indicates a relatively desirable breakpoint.
        cost: f32,
        /// Whether or not this is a flagged penalty item. Some algorithms will attempt to avoid
        /// having multiple consecutive breaks at flagged penalty items.
        flagged: bool,
    },
}

impl Item {
    fn penalty_cost(&self) -> f32 {
        match self {
            Item::Penalty { cost, .. } => *cost,
            _ => 0.0,
        }
    }

    fn penalty_flag(&self) -> f32 {
        match self {
            Item::Penalty { flagged, .. } => {
                if *flagged {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn is_mandatory_break(&self) -> bool {
        match self {
            Item::Penalty { cost, .. } => *cost == f32::NEG_INFINITY,
            _ => false,
        }
    }
}

/// A single line of text as represented by its break point and adjustment ratio.
#[derive(Debug, Default, Clone, Copy)]
pub struct Line {
    /// The index of the item at which to break this line.
    pub break_at: usize,
    /// The adjustment ratio that should be applied to glue when rendering this line. If the
    /// adjustment ratio is negative, glue should be adjusted by its shrink parameter. If the
    /// adjustment ratio is positive, glue should be adjusted by its stretch parameter. In general,
    pub adjustment_ratio: f32,
}

impl Line {
    /// Returns the width of a glue item with the given width, stretch, and shrink once the
    /// adjustment ratio is taken into account.
    pub fn glue_width(&self, width: f32, stretch: f32, shrink: f32) -> f32 {
        if self.adjustment_ratio < 0.0 {
            width + shrink * self.adjustment_ratio
        } else if self.adjustment_ratio > 0.0 {
            width + stretch * self.adjustment_ratio
        } else {
            width
        }
    }
}

/// Represents a paragraph layout algorithm
pub trait ParagraphLayout {
    /// Lays out a paragraph with the given line width that consists of as list of items and
    /// returns the laid-out lines.
    fn layout_paragraph(&self, items: &[Item], line_width: f32) -> Vec<Line>;
}
