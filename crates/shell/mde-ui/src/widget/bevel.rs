//! Windows 2000 3D edge model (the `DrawEdge` semantics).
//!
//! Every classic Win2000 control is a rectangle drawn with a two-line bevel:
//! an outer line and an inner line on each side. The exact color of each line
//! is what makes a control read as "raised" (button up), "sunken" (text field,
//! pressed button), or "window frame". These come straight from the system
//! 3D color ramp, so they track the palette automatically.
//!
//! This module is pure data — no iced calls — so it can be unit-tested and
//! reused by the renderer. The widget layer maps a [`Bevel`] onto four 1px
//! border quads.

use crate::palette::{Rgb, BUTTON_DK_SHADOW, BUTTON_HILIGHT, BUTTON_LIGHT, BUTTON_SHADOW};

/// The four bevel lines of a 3D edge. `tl` lines run along the top+left,
/// `br` lines along the bottom+right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bevel {
    pub outer_tl: Rgb,
    pub outer_br: Rgb,
    pub inner_tl: Rgb,
    pub inner_br: Rgb,
}

impl Bevel {
    /// `EDGE_RAISED` — a button at rest, the taskbar, panel faces.
    /// Outer: white / dark-shadow. Inner: light / shadow.
    pub const fn raised() -> Self {
        Self {
            outer_tl: BUTTON_HILIGHT,
            outer_br: BUTTON_DK_SHADOW,
            inner_tl: BUTTON_LIGHT,
            inner_br: BUTTON_SHADOW,
        }
    }

    /// `EDGE_SUNKEN` — text fields, list/tree views, a pressed button.
    /// The mirror image of [`raised`](Self::raised).
    pub const fn sunken() -> Self {
        Self {
            outer_tl: BUTTON_DK_SHADOW,
            outer_br: BUTTON_HILIGHT,
            inner_tl: BUTTON_SHADOW,
            inner_br: BUTTON_LIGHT,
        }
    }

    /// A pressed (depressed) button: a single sunken line, no inner highlight.
    pub const fn pressed() -> Self {
        Self {
            outer_tl: BUTTON_SHADOW,
            outer_br: BUTTON_LIGHT,
            inner_tl: BUTTON_SHADOW,
            inner_br: BUTTON_LIGHT,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raised_and_sunken_are_mirror_images() {
        let r = Bevel::raised();
        let s = Bevel::sunken();
        assert_eq!(r.outer_tl, s.outer_br);
        assert_eq!(r.outer_br, s.outer_tl);
        assert_eq!(r.inner_tl, s.inner_br);
        assert_eq!(r.inner_br, s.inner_tl);
    }
}
