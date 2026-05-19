"""Pure-helper tests for the Phase 12.9 topology renderer.

The math (seed_positions, relax_layout, hit_test_*, filter_for_node_view,
point_to_segment_distance) lives outside the GTK class so it runs
under the no-pytest shim and real pytest alike.
"""
from __future__ import annotations

import math


def test_seed_positions_places_every_node_inside_bounds():
    from mackes.workbench.network.mesh_topology_render import Node, seed_positions
    nodes = [Node(node_id=f"peer:{i}") for i in range(6)]
    seed_positions(nodes, 800.0, 600.0, seed=42)
    for n in nodes:
        assert 0 <= n.x <= 800, f"{n.node_id} x={n.x} out of bounds"
        assert 0 <= n.y <= 600, f"{n.node_id} y={n.y} out of bounds"


def test_seed_positions_is_deterministic_for_same_seed():
    from mackes.workbench.network.mesh_topology_render import Node, seed_positions
    a = [Node(node_id=f"peer:{i}") for i in range(4)]
    b = [Node(node_id=f"peer:{i}") for i in range(4)]
    seed_positions(a, 400.0, 300.0, seed=7)
    seed_positions(b, 400.0, 300.0, seed=7)
    for na, nb in zip(a, b):
        assert na.x == nb.x and na.y == nb.y


def test_seed_positions_handles_empty_list():
    from mackes.workbench.network.mesh_topology_render import seed_positions
    seed_positions([], 100.0, 100.0)  # no crash


def test_relax_layout_keeps_nodes_inside_bounds():
    from mackes.workbench.network.mesh_topology_render import (
        Node, Edge, seed_positions, relax_layout,
    )
    nodes = [Node(node_id=f"peer:{i}") for i in range(5)]
    seed_positions(nodes, 500.0, 400.0, seed=1)
    edges = [Edge(a="peer:0", b="peer:1"), Edge(a="peer:1", b="peer:2")]
    relax_layout(nodes, edges, 500.0, 400.0, iterations=20)
    for n in nodes:
        assert 0 <= n.x <= 500
        assert 0 <= n.y <= 400


def test_relax_layout_is_idempotent_in_steady_state():
    """Re-running relaxation on an already-converged layout shouldn't
    move nodes significantly — the steady-state invariant."""
    from mackes.workbench.network.mesh_topology_render import (
        Node, Edge, seed_positions, relax_layout,
    )
    nodes = [Node(node_id=f"peer:{i}") for i in range(5)]
    seed_positions(nodes, 600.0, 500.0, seed=11)
    edges = [Edge(a="peer:0", b="peer:1"), Edge(a="peer:2", b="peer:3")]
    # First converge.
    relax_layout(nodes, edges, 600.0, 500.0, iterations=200)
    before = [(n.x, n.y) for n in nodes]
    # One more pass.
    relax_layout(nodes, edges, 600.0, 500.0, iterations=5)
    after = [(n.x, n.y) for n in nodes]
    for (ax, ay), (bx, by) in zip(before, after):
        # Allow some drift but not "still chaotic" levels.
        assert math.hypot(ax - bx, ay - by) < 50.0


def test_hit_test_node_returns_closest():
    from mackes.workbench.network.mesh_topology_render import Node, hit_test_node
    nodes = {
        "a": Node(node_id="a", x=100, y=100),
        "b": Node(node_id="b", x=200, y=100),
    }
    hit = hit_test_node(nodes, 105, 100, radius=20)
    assert hit is not None and hit.node_id == "a"


def test_hit_test_node_returns_none_when_far():
    from mackes.workbench.network.mesh_topology_render import Node, hit_test_node
    nodes = {"a": Node(node_id="a", x=100, y=100)}
    hit = hit_test_node(nodes, 500, 500, radius=20)
    assert hit is None


def test_hit_test_edge_finds_midpoint():
    from mackes.workbench.network.mesh_topology_render import (
        Layout, Node, Edge, hit_test_edge,
    )
    layout = Layout(
        nodes={"a": Node(node_id="a", x=0, y=0),
               "b": Node(node_id="b", x=100, y=0)},
        edges=[Edge(a="a", b="b")],
    )
    edge = hit_test_edge(layout, 50, 2, tolerance=6)
    assert edge is not None and edge.a == "a" and edge.b == "b"


def test_hit_test_edge_misses_when_far():
    from mackes.workbench.network.mesh_topology_render import (
        Layout, Node, Edge, hit_test_edge,
    )
    layout = Layout(
        nodes={"a": Node(node_id="a", x=0, y=0),
               "b": Node(node_id="b", x=100, y=0)},
        edges=[Edge(a="a", b="b")],
    )
    assert hit_test_edge(layout, 50, 50, tolerance=6) is None


def test_point_to_segment_distance_on_segment_is_zero():
    from mackes.workbench.network.mesh_topology_render import (
        point_to_segment_distance,
    )
    # Midpoint of (0,0)-(100,0) -> distance 0.
    assert point_to_segment_distance(50, 0, 0, 0, 100, 0) == 0


def test_point_to_segment_distance_off_segment_uses_perpendicular():
    from mackes.workbench.network.mesh_topology_render import (
        point_to_segment_distance,
    )
    # 50 above the midpoint of (0,0)-(100,0).
    assert point_to_segment_distance(50, 50, 0, 0, 100, 0) == 50.0


def test_point_to_segment_distance_clamps_to_endpoints():
    from mackes.workbench.network.mesh_topology_render import (
        point_to_segment_distance,
    )
    # Point to the LEFT of segment (0,0)-(100,0) — closest is endpoint.
    d = point_to_segment_distance(-50, 0, 0, 0, 100, 0)
    assert d == 50.0


def test_point_to_segment_distance_degenerate_segment_is_endpoint_distance():
    from mackes.workbench.network.mesh_topology_render import (
        point_to_segment_distance,
    )
    # Zero-length segment at (10,10), query at (13,14) -> distance 5.
    d = point_to_segment_distance(13, 14, 10, 10, 10, 10)
    assert d == 5.0


def test_filter_for_node_view_keeps_focus_plus_direct_neighbors():
    from mackes.workbench.network.mesh_topology_render import (
        Layout, Node, Edge, filter_for_node_view,
    )
    layout = Layout(
        nodes={
            "a": Node(node_id="a"),
            "b": Node(node_id="b"),
            "c": Node(node_id="c"),
            "d": Node(node_id="d"),
        },
        edges=[
            Edge(a="a", b="b"),
            Edge(a="a", b="c"),
            Edge(a="c", b="d"),  # neighbor-of-neighbor, must be dropped
        ],
    )
    sub = filter_for_node_view(layout, "a")
    assert set(sub.nodes) == {"a", "b", "c"}
    assert len(sub.edges) == 2
    # The (c,d) edge must be filtered out because d isn't in the view.
    assert all(not (e.a == "c" and e.b == "d") for e in sub.edges)


def test_filter_for_node_view_returns_empty_for_unknown_focus():
    from mackes.workbench.network.mesh_topology_render import (
        Layout, Node, Edge, filter_for_node_view,
    )
    layout = Layout(
        nodes={"a": Node(node_id="a")},
        edges=[],
    )
    sub = filter_for_node_view(layout, "does-not-exist")
    assert sub.nodes == {}
    assert sub.edges == []
