extern crate alloc;
use alloc::{vec, vec::Vec};
use bumpalo::Bump;

use crate::math::Num;
use crate::{Item, Line, ParagraphLayout};

/// Runs the Knuth-Plass line-breaking algorithm to calculate the optimal break points for a
/// paragraph.
pub struct KnuthPlass<N> {
    flagged_demerit: N,
    fitness_demerit: N,
    threshold: N,
    looseness: usize,
}

impl<N: Num> KnuthPlass<N> {
    /// Creates a new KnuthPlass layout with default parameter values.
    pub fn new() -> Self {
        KnuthPlass {
            flagged_demerit: N::from(100),
            fitness_demerit: N::from(100),
            threshold: N::from(1),
            looseness: 0,
        }
    }

    /// Sets the demerit for flagged penalties. Defaults to 100. Referred to as 𝛂 in Knuth-Plass
    /// '81.
    pub fn with_flagged_demerit(mut self, flagged_demerit: N) -> Self {
        self.flagged_demerit = flagged_demerit;
        self
    }

    /// Sets the demerit for a line that belongs to a different fitness class than its predecessor.
    /// Defaults to 100. Referred to as 𝛄 in Knuth-Plass '81.
    pub fn with_fitness_demerit(mut self, fitness_demerit: N) -> Self {
        self.fitness_demerit = fitness_demerit;
        self
    }

    /// Sets the adjustment ratio threshold. Lines will not be allowed to break at a given point if
    /// doing so would cause the line's adjustment ratio to exceed this value. Defaults to 1.
    /// Referred to as 𝛒 in Knuth-Plass '81.
    pub fn with_threshold(mut self, threshold: N) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sets the looseness parameter. The looseness is an integer 𝗾 such that the total number of
    /// lines produced for the paragraph is as close as possible to 𝗾 plus the optimum number,
    /// without violating the conditions of feasibility.
    pub fn with_looseness(mut self, looseness: usize) -> Self {
        self.looseness = looseness;
        self
    }
}

impl<N: Num> Default for KnuthPlass<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Box, Glue, Penalty, N: Num> ParagraphLayout<Box, Glue, Penalty, N> for KnuthPlass<N> {
    fn layout_paragraph(
        &self,
        items: &[Item<Box, Glue, Penalty, N>],
        line_width: N,
    ) -> Vec<Line<N>> {
        let layout = KnuthPlassLayout {
            bump: Bump::new(),
            items,
            line_width,
            flagged_demerit: self.flagged_demerit,
            fitness_demerit: self.fitness_demerit,
            threshold: self.threshold,
            looseness: self.looseness,
            first_uniform_line: 0,
            total_width: N::from(0),
            total_stretch: N::from(0),
            total_shrink: N::from(0),
            active: None,
        };
        unsafe { layout.run() }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Fitness {
    #[default]
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl Fitness {
    fn distance(&self, other: &Fitness) -> usize {
        (*self as isize - *other as isize).unsigned_abs()
    }
}

/// A Node tracks a feasible line break.
#[derive(Default)]
struct Node<N> {
    /// The position of the line break within the paragraph.
    position: usize,
    /// The index of the line that terminates at this break.
    line: usize,
    /// The break's fitness class.
    fitness: Fitness,
    /// 𝚺𝓌 after position per Knuth-Plass '81.
    total_width: N,
    /// 𝚺𝓎 after position per Knuth-Plass '81.
    total_stretch: N,
    /// 𝚺𝓏 after position per Knuth-Plass '81.
    total_shrink: N,
    /// Minimum total demerits up to this break point.
    total_demerits: N,
    /// Pointer to the best node for the preceeding break point.
    previous: Option<*mut Node<N>>,
    /// Pointer to the next active node.
    link: Option<*mut Node<N>>,
}

/// Holder for the state used by Knuth-Plass. Tracks various configuration parameters plus the
/// running width, stretch, shrink, and active node.
///
/// Active nodes are allocated using a bump allocator and deallocated en masse once the algorithm
/// terminates.
struct KnuthPlassLayout<'a, Box, Glue, Penalty, N> {
    /// Allocator for break nodes.
    bump: Bump,

    /// The paragraph's items.
    items: &'a [Item<Box, Glue, Penalty, N>],
    /// The line width parameter.
    line_width: N,

    /// Demerit for flagged penalties. Referred to as 𝛂 in Knuth-Plass '81.
    flagged_demerit: N,
    /// Demerit for differing fitness classes. Referred to as 𝛄 in Knuth-Plass '81.
    fitness_demerit: N,
    /// Adjustment ratio threshold.  Referred to as 𝛒 in Knuth-Plass '81.
    threshold: N,
    /// Looseness parameter. Referred to as 𝗾 in Knuth-Plass '81.
    looseness: usize,
    /// Index of the first line that begins a block of uniformly-long lines that extends to the end
    /// of the paragraph. 𝒿₀ in Knuth-Plass '81.
    first_uniform_line: usize,

    /// Total width of all items in the paragraph up to the current item.
    total_width: N,
    /// Total stretch of all items in the paragraph up to the current item.
    total_stretch: N,
    /// Total shrink of all items in the paragraph up to the current item.
    total_shrink: N,
    /// Head of the linked list of active nodes.
    active: Option<*mut Node<N>>,
}

impl<'a, Box, Glue, Penalty, N: Num> KnuthPlassLayout<'a, Box, Glue, Penalty, N> {
    /// Creates a new node for a breakpoint. Currently just a wrapper for bump.alloc.
    fn new_node(&mut self, node: Node<N>) -> *mut Node<N> {
        self.bump.alloc(node)
    }

    /// Placeholder method for determining the width of a given line. Currently just returns
    /// line_width.
    fn get_line_width(&self, _l: usize) -> N {
        self.line_width
    }

    /// Returns the width, stretch, and shrink of the node at b and indicates whether or not b is a
    /// legal break.
    fn is_legal_breakpoint(&self, b: usize) -> (N, N, N, bool) {
        self.items[b].is_legal_breakpoint((b != 0).then(|| &self.items[b - 1]))
    }

    /// Calculates the line number and adjustment ratio for a line from the end of a to b.
    fn adjustment_ratio(&self, a: &Node<N>, b: usize) -> (usize, N) {
        let j = a.line + 1;
        let r = self.items[b].adjustment_ratio(
            self.total_width - a.total_width,
            self.total_stretch - a.total_stretch,
            self.total_shrink - a.total_shrink,
            self.get_line_width(j),
        );
        (j, r)
    }

    /// Deactivates the given node by removing it from the active list.
    unsafe fn deactivate_node(&mut self, a: &mut Node<N>) {
        if let Some(previous) = a.previous {
            (*previous).link = a.link;
        }
        if self.active == Some(a) {
            self.active = a.link;
        }
    }

    /// Calculates the demerits and fitness class for a line from a to b.
    unsafe fn demerits_and_fitness(&self, r: N, a: &Node<N>, b: usize) -> (N, Fitness) {
        let cost = self.items[b].penalty_cost();
        let d = if cost >= N::from(0) {
            (N::from(1) + N::from(100) * r.abs().powi(3) + cost).powi(2)
        } else if cost != N::NEG_INFINITY {
            (N::from(1) + N::from(100) * r.abs().powi(3)).powi(2) - cost.powi(2)
        } else {
            (N::from(1) + N::from(100) * r.abs().powi(3)).powi(2)
        };
        let d = d + self.flagged_demerit
            * self.items[b].penalty_flag()
            * self.items[a.position].penalty_flag();

        let c = if r < N::rat(-1, 2) {
            Fitness::Zero
        } else if r <= N::rat(1, 2) {
            Fitness::One
        } else if r <= N::from(1) {
            Fitness::Two
        } else {
            Fitness::Three
        };

        let d = if c.distance(&a.fitness) > 1 {
            d + self.fitness_demerit
        } else {
            d
        };
        (d + a.total_demerits, c)
    }

    /// Calculates 𝚺𝓌 after b, 𝚺𝓎 after b, and 𝚺𝓏 after b per Knuth-Plass '81.
    fn total_after(&self, b: usize) -> (N, N, N) {
        let (mut total_width, mut total_stretch, mut total_shrink) =
            (self.total_width, self.total_stretch, self.total_shrink);
        for i in b..self.items.len() {
            match self.items[i] {
                Item::Box { .. } => break,
                Item::Glue {
                    width,
                    stretch,
                    shrink,
                    ..
                } => {
                    total_width += width;
                    total_stretch += stretch;
                    total_shrink += shrink;
                }
                Item::Penalty { cost, .. } => {
                    if cost == N::NEG_INFINITY && i > b {
                        break;
                    }
                }
            };
        }
        (total_width, total_stretch, total_shrink)
    }

    /// Main loop for processing a legal breakpoint. Returns false if no layout is possible.
    unsafe fn layout_breakpoint(&mut self, b: usize) -> bool {
        let mut a = self.active;
        let mut prev_a = None;
        while a.is_some() {
            let mut class_a: [Option<*mut Node<N>>; 4] = [None, None, None, None];
            let mut class_demerits: [N; 4] = [N::INFINITY, N::INFINITY, N::INFINITY, N::INFINITY];
            let mut min_demerits: N = N::INFINITY;
            loop {
                let unwrapped_a = &mut *a.unwrap();
                let next_a = unwrapped_a.link;

                let (j, r) = self.adjustment_ratio(unwrapped_a, b);
                if r < N::from(-1) || self.items[b].is_mandatory_break() {
                    self.deactivate_node(unwrapped_a);
                } else {
                    prev_a = a;
                }
                if N::from(-1) <= r && r <= self.threshold {
                    let (demerits, fitness) = self.demerits_and_fitness(r, unwrapped_a, b);
                    if demerits < class_demerits[fitness as usize] {
                        class_demerits[fitness as usize] = demerits;
                        class_a[fitness as usize] = a;
                        if demerits < min_demerits {
                            min_demerits = demerits;
                        }
                    }
                }

                a = next_a;
                match a {
                    None => break,
                    Some(a) => {
                        if (*a).line >= j && j < self.first_uniform_line {
                            break;
                        }
                    }
                };
            }
            if min_demerits < N::INFINITY {
                let (total_width, total_stretch, total_shrink) = self.total_after(b);
                let min_demerits = min_demerits + self.fitness_demerit;
                for c in [Fitness::Zero, Fitness::One, Fitness::Two, Fitness::Three] {
                    let demerits = class_demerits[c as usize];
                    if demerits <= min_demerits {
                        let class_a = class_a[c as usize].unwrap();
                        let s = self.new_node(Node {
                            position: b,
                            line: (*class_a).line + 1,
                            fitness: c,
                            total_width,
                            total_stretch,
                            total_shrink,
                            total_demerits: demerits,
                            previous: Some(class_a),
                            link: a,
                        });
                        match prev_a {
                            None => self.active = Some(s),
                            Some(prev_a) => (*prev_a).link = Some(s),
                        };
                        prev_a = Some(s);
                    }
                }
            }
        }
        self.active.is_some()
    }

    /// Driver for Knuth-Plass paragraph layout.
    unsafe fn run(mut self) -> Vec<Line<N>> {
        // Initialize the list of active nodes.
        self.active = Some(self.new_node(Default::default()));

        // Loop over the items to lay out and calculate the set of legal breakpoints.
        for b in 0..self.items.len() {
            let (width, stretch, shrink, is_legal) = self.is_legal_breakpoint(b);
            if is_legal && !self.layout_breakpoint(b) {
                return Vec::new();
            }
            self.total_width += width;
            self.total_stretch += stretch;
            self.total_shrink += shrink;
        }
        if self.active.is_none() {
            return Vec::new();
        }

        // Choose the active node with the fewest demerits.
        let mut a = self.active;
        let mut b = &*a.unwrap();
        loop {
            match a {
                None => break,
                Some(n) => {
                    let n = &*n;
                    if n.total_demerits < b.total_demerits {
                        b = n;
                    }
                    a = n.link;
                }
            };
        }

        // Choose the appropriate active node.
        if self.looseness != 0 {
            let k = b.line;

            let mut a = &*self.active.unwrap();
            let mut b = a;
            let mut s = 0;
            loop {
                let delta = a.line - k;
                if self.looseness <= delta && delta < s || s < delta && delta <= self.looseness {
                    s = delta;
                    b = a;
                } else if delta == s && a.total_demerits < b.total_demerits {
                    b = a;
                }
                match a.link {
                    None => break,
                    Some(link) => a = &*link,
                };
            }
        };

        // Walk backwards from the chosen node to the start of the paragraph to compute the chosen
        // line breaks.
        let mut lines = vec![Default::default(); b.line];
        let mut j = b.line;
        while j > 0 {
            let prev = &*b.previous.unwrap();
            let prev_pos = if j == 1 { 0 } else { prev.position + 1 };

            let items = &self.items[prev_pos..b.position];
            let (width, stretch, shrink) = items
                .iter()
                .map(|item| match item {
                    Item::Box { width, .. } => (*width, N::from(0), N::from(0)),
                    Item::Glue {
                        width,
                        stretch,
                        shrink,
                        ..
                    } => (*width, *stretch, *shrink),
                    Item::Penalty { width, .. } => (*width, N::from(0), N::from(0)),
                })
                .reduce(|acc, n| (acc.0 + n.0, acc.1 + n.1, acc.2 + n.2))
                .unwrap();

            let at = &self.items[b.position];
            let line_width = self.get_line_width(j);
            let adjustment_ratio = at.adjustment_ratio(width, stretch, shrink, line_width);

            lines[j - 1] = Line {
                break_at: b.position,
                adjustment_ratio,
            };

            b = prev;
            j -= 1;
        }

        lines
    }
}
