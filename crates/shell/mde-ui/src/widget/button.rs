//! A Windows 2000 push button for iced.
//!
//! Raised 3D bevel at rest; when pressed it flips to a sunken bevel and the
//! label nudges 1px down-right — exactly like a classic Win2000 button. Used
//! for the Start button, taskbar window buttons, and dialog buttons.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{event, mouse};
use iced::{Color, Element, Event, Length, Padding, Rectangle, Size, Vector};

use crate::palette;
use crate::widget::{draw_edge, fill};

#[derive(Default)]
struct State {
    is_pressed: bool,
}

/// A classic 3D push button wrapping arbitrary content (usually text).
pub struct Button<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    face: Color,
    /// When true the button renders sunken even when not pressed (toggled on,
    /// e.g. the focused window's taskbar button).
    active: bool,
    /// The default button in a dialog: drawn with an extra 1px black outline,
    /// activated by Enter.
    default: bool,
}

/// Construct a button around `content`.
pub fn button<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    Button {
        content: content.into(),
        on_press: None,
        width: Length::Shrink,
        height: Length::Shrink,
        padding: Padding::from([2, 8]),
        face: palette::color(palette::BUTTON_FACE),
        active: false,
        default: false,
    }
}

impl<'a, Message, Theme, Renderer> Button<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
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
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    pub fn default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Button<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            // Loose limits (min 0) so the content node is its intrinsic size, not
            // stretched to fill a fixed-height button — `draw` then vertically
            // centers it (Carbon button heights, E9.3).
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, &limits.loose())
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let enabled = self.on_press.is_some();
        // A pressed or toggled-on (active) control nudges its label 1px down-right —
        // the flat Carbon press feedback (the 3D sunken bevel is gone, E9.7).
        let down = state.is_pressed || self.active;
        let b = layout.bounds();
        if self.default {
            // The default button: a 1px frame outline around the flat face.
            let black = palette::color(palette::WINDOW_FRAME);
            fill(renderer, b.x, b.y, b.width, 1.0, black);
            fill(renderer, b.x, b.y, 1.0, b.height, black);
            fill(renderer, b.x, b.y + b.height - 1.0, b.width, 1.0, black);
            fill(renderer, b.x + b.width - 1.0, b.y, 1.0, b.height, black);
            draw_edge(
                renderer,
                Rectangle {
                    x: b.x + 1.0,
                    y: b.y + 1.0,
                    width: b.width - 2.0,
                    height: b.height - 2.0,
                },
                Some(self.face),
            );
        } else {
            draw_edge(renderer, b, Some(self.face));
        }

        let content_layout = layout.children().next().expect("button has one child");
        // A button with no action is disabled: gray (embossed) label, the way
        // Win2000 drew COLOR_GRAYTEXT. An actionable button uses BUTTON_TEXT.
        let content_style = renderer::Style {
            text_color: palette::color(if enabled {
                palette::BUTTON_TEXT
            } else {
                palette::GRAY_TEXT
            }),
        };
        // Vertically center the label when the button is taller than its content
        // (Carbon button heights, E9.3 — `.height(BUTTON_MD)` etc.). The layout
        // places the content at `padding.top`; shift it down by the remaining slack
        // so a 40/48px button centers its label. A content-sized (Shrink) button has
        // no slack, so this is a no-op there. Hit-testing uses the full button
        // bounds, so moving the label in `draw` only is safe.
        let cb = content_layout.bounds();
        let center_y = (((b.height - cb.height) / 2.0) - (cb.y - b.y)).max(0.0);
        let nudge = if down { 1.0 } else { 0.0 };
        let offset = Vector::new(nudge, nudge + center_y);
        renderer.with_translation(offset, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &content_style,
                content_layout,
                cursor,
                viewport,
            );
        });
        let _ = style;
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(layout.bounds()) {
                    state.is_pressed = true;
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.is_pressed => {
                state.is_pressed = false;
                if cursor.is_over(layout.bounds()) {
                    if let Some(message) = self.on_press.clone() {
                        shell.publish(message);
                    }
                    return event::Status::Captured;
                }
            }
            _ => {}
        }
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Button<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(button: Button<'a, Message, Theme, Renderer>) -> Self {
        Self::new(button)
    }
}
