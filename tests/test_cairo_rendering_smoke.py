"""Cairo rendering smoke test (Phase 12.11.4).

Renders the topology renderer's paint logic to a headless
``cairo.ImageSurface`` (no Xvfb / no GTK display required) and
asserts the output carries expected ink in expected color buckets:

  * healthy nodes contribute green pixels
  * edges contribute blue pixels
  * the canvas background is not entirely empty

This is not a full snapshot diff — those land alongside CI's
Xvfb-driven E2E suite. The intent here is a regression guard
catching trivial breakages (constants flipped, layout function
returning nothing, color tuple corruption) without needing a real
display.
"""
from __future__ import annotations

import math


def _skip_if_no_cairo():
    try:
        import cairo  # noqa: F401
    except ImportError:
        import pytest
        pytest.skip("pycairo not installed; Cairo smoke test skipped")


def _make_layout():
    from mackes.workbench.network.mesh_topology_render import (
        Node, Edge, Layout,
    )
    nodes = {
        "a": Node(node_id="a", label="a", health="healthy", x=120, y=120),
        "b": Node(node_id="b", label="b", health="degraded", x=320, y=120),
        "c": Node(node_id="c", label="c", health="unreachable", x=220, y=300),
    }
    edges = [
        Edge(a="a", b="b", state="healthy"),
        Edge(a="b", b="c", state="missing"),
        Edge(a="a", b="c", state="extra"),
    ]
    return Layout(nodes=nodes, edges=edges)


def _paint_layout(cr, layout, width, height) -> None:
    """Direct port of MeshTopologyRender._on_draw — copy here so we
    don't depend on the full GTK widget."""
    from mackes.workbench.network.mesh_topology_render import (
        _HEALTH_FILL, _EDGE_COLOR,
    )
    cr.set_source_rgb(0.07, 0.07, 0.09)
    cr.rectangle(0, 0, width, height)
    cr.fill()
    for e in layout.edges:
        if e.a not in layout.nodes or e.b not in layout.nodes:
            continue
        a = layout.nodes[e.a]
        b = layout.nodes[e.b]
        r, g, b_c = _EDGE_COLOR.get(e.state, (0.5, 0.5, 0.5))
        cr.set_source_rgb(r, g, b_c)
        cr.set_line_width(2.0)
        cr.move_to(a.x, a.y)
        cr.line_to(b.x, b.y)
        cr.stroke()
    for n in layout.nodes.values():
        r, g, b_c = _HEALTH_FILL.get(n.health, _HEALTH_FILL["unknown"])
        cr.set_source_rgb(r, g, b_c)
        cr.arc(n.x, n.y, 12.0, 0, 2 * math.pi)
        cr.fill()


def _sample_pixel(surface, x, y):
    """Read an ARGB pixel out of a cairo ImageSurface. Returns
    (r, g, b) as 0..255 ints."""
    data = surface.get_data()
    stride = surface.get_stride()
    offset = y * stride + x * 4
    # Cairo ARGB32 is BGRA in memory on little-endian.
    b = data[offset]; g = data[offset + 1]; r = data[offset + 2]
    return (r, g, b)


def test_topology_renderer_paints_healthy_node_in_green():
    _skip_if_no_cairo()
    import cairo
    width, height = 440, 400
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, width, height)
    cr = cairo.Context(surface)
    layout = _make_layout()
    _paint_layout(cr, layout, width, height)
    # Center of the healthy node ("a" at 120,120) should be green-
    # dominant.
    r, g, b = _sample_pixel(surface, 120, 120)
    assert g > r and g > b, f"healthy node should be green-dominant, got rgb=({r},{g},{b})"


def test_topology_renderer_paints_degraded_node_in_amber():
    _skip_if_no_cairo()
    import cairo
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, 440, 400)
    cr = cairo.Context(surface)
    _paint_layout(cr, _make_layout(), 440, 400)
    # Degraded node "b" at (320,120) should be R+G dominant (amber).
    r, g, b = _sample_pixel(surface, 320, 120)
    assert r > b and g > b, f"degraded node should be amber, got rgb=({r},{g},{b})"


def test_topology_renderer_paints_unreachable_node_in_red():
    _skip_if_no_cairo()
    import cairo
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, 440, 400)
    cr = cairo.Context(surface)
    _paint_layout(cr, _make_layout(), 440, 400)
    # Unreachable node "c" at (220,300) should be red-dominant.
    r, g, b = _sample_pixel(surface, 220, 300)
    assert r > g and r > b, f"unreachable node should be red, got rgb=({r},{g},{b})"


def test_topology_renderer_background_is_dark():
    _skip_if_no_cairo()
    import cairo
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, 440, 400)
    cr = cairo.Context(surface)
    _paint_layout(cr, _make_layout(), 440, 400)
    # Top-left corner is background; expect low intensity in all channels.
    r, g, b = _sample_pixel(surface, 2, 2)
    assert r < 60 and g < 60 and b < 60, (
        f"background should be near-black, got rgb=({r},{g},{b})"
    )


def test_topology_renderer_paints_at_least_one_blue_edge_pixel():
    _skip_if_no_cairo()
    import cairo
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, 440, 400)
    cr = cairo.Context(surface)
    _paint_layout(cr, _make_layout(), 440, 400)
    # Sample the midpoint of the healthy edge from a(120,120) to
    # b(320,120) -> (220,120). Should hit the blue edge.
    r, g, b = _sample_pixel(surface, 220, 120)
    assert b > r and b > g, (
        f"healthy-edge midpoint should be blue-dominant, got rgb=({r},{g},{b})"
    )
