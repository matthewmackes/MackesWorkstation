"""Shared widget helpers used across workbench panels.

Keeping these in one place so every panel renders consistently — section
headers, two-column rows, scroll wrappers — without each panel growing its
own boilerplate.
"""
from __future__ import annotations

import gi
gi.require_version("Gtk", "3.0")
from gi.repository import Gtk  # noqa: E402


LABEL_COL_WIDTH = 180


def section_header(title: str) -> Gtk.Widget:
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
    lbl = Gtk.Label(label=title.upper())
    lbl.set_xalign(0)
    ctx = lbl.get_style_context()
    ctx.add_class("title-3")
    ctx.add_class("mackes-section-header")
    box.pack_start(lbl, False, False, 0)
    return box


def labeled_row(label_text: str, widget: Gtk.Widget) -> Gtk.Widget:
    row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
    lbl = Gtk.Label(label=label_text)
    lbl.set_xalign(0)
    lbl.set_size_request(LABEL_COL_WIDTH, -1)
    lbl.get_style_context().add_class("mackes-row-label")
    row.pack_start(lbl, False, False, 0)
    row.pack_start(widget, True, True, 0)
    return row


def panel_box(margin: int = 24) -> Gtk.Box:
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=20)
    box.set_margin_top(margin); box.set_margin_bottom(margin)
    box.set_margin_start(margin + 4); box.set_margin_end(margin + 4)
    return box


def info_label(text: str) -> Gtk.Widget:
    lbl = Gtk.Label(label=text)
    lbl.set_xalign(0)
    lbl.set_line_wrap(True)
    ctx = lbl.get_style_context()
    ctx.add_class("dim-label")
    ctx.add_class("mackes-info")
    return lbl


def error_label(text: str) -> Gtk.Widget:
    lbl = Gtk.Label(label=text)
    lbl.set_xalign(0)
    lbl.set_line_wrap(True)
    lbl.get_style_context().add_class("error")
    return lbl


def title_label(text: str) -> Gtk.Widget:
    lbl = Gtk.Label(label=text)
    lbl.set_xalign(0)
    ctx = lbl.get_style_context()
    ctx.add_class("title-2")
    ctx.add_class("mackes-panel-title")
    return lbl
