//! A childless flat Carbon frame widget for iced.
//!
//! Fills its bounds with the face color and draws a single 1px subtle border at a
//! 2px radius (via [`crate::widget::draw_edge`]). Being childless it is trivially
//! correct and composes as a background in a `stack!` or as a separator /
//! group-box / clock-well frame. (The Win2000 raised/sunken/pressed 3D bevel was
//! retired in the Carbon-only collapse, E9.7; `raised`/`sunken` and `thickness` are
//! kept as no-op-distinct builders so existing call sites are unchanged.)

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::mouse;
use iced::{Color, Element, Length, Rectangle, Size};

use crate::palette;
use crate::widget::draw_edge;

/// A flat Carbon frame. See [`raised`], [`sunken`].
pub struct BevelFrame {
    face: Option<Color>,
    width: Length,
    height: Length,
}

/// A frame for panels, the taskbar, buttons at rest. (Flat under Carbon.)
pub fn raised() -> BevelFrame {
    BevelFrame::new()
}
/// A frame for text fields, list/tree views, the clock well. (Flat under Carbon.)
pub fn sunken() -> BevelFrame {
    BevelFrame::new()
}
impl BevelFrame {
    fn new() -> Self {
        Self {
            face: Some(palette::color(palette::BUTTON_FACE)),
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    pub fn face(mut self, color: Color) -> Self {
        self.face = Some(color);
        self
    }
    pub fn no_face(mut self) -> Self {
        self.face = None;
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    /// No-op kept for call-site compatibility: the flat Carbon edge is a single 1px
    /// line regardless of thickness (the 3D two-line edge is gone, E9.7).
    pub fn thickness(self, _thickness: u16) -> Self {
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for BevelFrame
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        draw_edge(renderer, layout.bounds(), self.face);
    }
}

impl<'a, Message, Theme, Renderer> From<BevelFrame> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
{
    fn from(frame: BevelFrame) -> Self {
        Self::new(frame)
    }
}
