//! Carbon property-sheet tab control (E9.3).
//!
//! Flat tabs: each tab is a face fill with a single 1px subtle border (the 3D
//! Win2000 white/silver/gray bevels were retired in the Carbon-only collapse,
//! E9.7). Two cues carry the state: the *seam* — an inactive tab keeps its base
//! border (it sits above the page) while the selected tab omits it, so it merges
//! into the page body beneath — and a strict 2px Carbon accent bar along the top
//! of the selected tab (E9.4 interaction states).
//!
//! [`tab_strip`] builds the whole header row; the caller stacks the page body
//! directly below it, exactly as the real dialogs do.

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Tree, Widget};
use iced::mouse;
use iced::widget::{container, mouse_area, text, Row};
use iced::{Element, Length, Padding, Rectangle, Size};

use crate::palette;
use crate::widget::fill;
use crate::{font, metrics};

/// Tab header height (SM_CYCAPTION-ish; the classic tab is ~20px at 96 DPI).
const TAB_H: f32 = 20.0;

/// A childless widget that draws one tab's 3D edge. `active` drops the base
/// shadow line so the tab connects to the page below.
struct TabEdge {
    active: bool,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for TabEdge
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
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
        let b = layout.bounds();
        let face = palette::color(palette::BUTTON_FACE);
        let border = palette::color(palette::WINDOW_FRAME);

        // Flat Carbon tab (E9.3): a face fill with a single 1px subtle border —
        // no 3D bevel (the raised Win2000 white/silver/gray edges were retired in
        // the Carbon-only collapse, E9.7; the same flat treatment as `draw_edge`).
        fill(renderer, b.x, b.y, b.width, b.height, face);
        fill(renderer, b.x, b.y, b.width, 1.0, border); // top
        fill(renderer, b.x, b.y, 1.0, b.height, border); // left
        fill(renderer, b.x + b.width - 1.0, b.y, 1.0, b.height, border); // right
                                                                         // Base line / seam: an inactive tab keeps its bottom border (it sits above
                                                                         // the page); the active tab omits it so it merges into the page body below.
        if !self.active {
            fill(renderer, b.x, b.y + b.height - 1.0, b.width, 1.0, border);
        }
        // E9.4 — Carbon selected-tab cue: a strict 2px accent bar along the top of
        // the active tab (the Carbon "selected" indicator). Inactive tabs: none.
        if self.active {
            fill(renderer, b.x, b.y, b.width, 2.0, palette::accent());
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TabEdge> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
{
    fn from(edge: TabEdge) -> Self {
        Self::new(edge)
    }
}

/// One clickable tab: the edge behind a centered label. The selected tab's
/// label nudges up 1px (it sits slightly proud, like the real control).
fn tab<'a, Message: Clone + 'a>(label: &str, active: bool, msg: Message) -> Element<'a, Message> {
    let pad = if active {
        Padding {
            top: 2.0,
            right: 10.0,
            bottom: 3.0,
            left: 10.0,
        }
    } else {
        Padding {
            top: 3.0,
            right: 10.0,
            bottom: 2.0,
            left: 10.0,
        }
    };
    let labelled = container(
        text(label.to_string())
            .size(metrics::UI_PX)
            .font(font::ui()),
    )
    .padding(pad)
    .height(Length::Fixed(TAB_H));
    mouse_area(iced::widget::stack![TabEdge { active }, labelled])
        .on_press(msg)
        .into()
}

/// The property-sheet tab header row. `selected` is highlighted as the merged
/// tab; clicking tab `i` emits `on_select(i)`. Stack the page body directly
/// below the returned element.
pub fn tab_strip<'a, Message: Clone + 'a>(
    labels: &[&str],
    selected: usize,
    on_select: impl Fn(usize) -> Message,
) -> Element<'a, Message> {
    let mut row = Row::new().spacing(2.0).padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 4.0,
    });
    for (i, l) in labels.iter().enumerate() {
        row = row.push(tab(l, i == selected, on_select(i)));
    }
    row.height(Length::Fixed(TAB_H)).into()
}
