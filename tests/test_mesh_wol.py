"""mesh_wol — MAC parsing + magic-packet construction."""
from __future__ import annotations

import socket


def test_normalise_mac_accepts_colon_form():
    from mackes.mesh_wol import _normalise_mac
    assert _normalise_mac("aa:bb:cc:dd:ee:ff") == bytes.fromhex("aabbccddeeff")


def test_normalise_mac_accepts_hyphen_form():
    from mackes.mesh_wol import _normalise_mac
    assert _normalise_mac("aa-bb-cc-dd-ee-ff") == bytes.fromhex("aabbccddeeff")


def test_normalise_mac_accepts_bare_hex():
    from mackes.mesh_wol import _normalise_mac
    assert _normalise_mac("aabbccddeeff") == bytes.fromhex("aabbccddeeff")


def test_normalise_mac_rejects_garbage():
    from mackes.mesh_wol import _normalise_mac
    assert _normalise_mac("not-a-mac") is None
    assert _normalise_mac("") is None
    assert _normalise_mac("aa:bb:cc:dd:ee") is None     # 5 octets
    assert _normalise_mac("aa:bb:cc:dd:ee:ff:00") is None  # 7 octets
    assert _normalise_mac("zz:bb:cc:dd:ee:ff") is None  # non-hex


def test_wake_returns_false_for_invalid_mac():
    from mackes.mesh_wol import wake
    assert wake("not-a-mac") is False


def test_wake_constructs_correct_magic_packet(tmp_path, monkeypatch):
    """Verify the bytes sent match the RFC 2965-ish magic-packet
    format: 6 × 0xFF then 16 × the destination MAC."""
    from mackes import mesh_wol
    sent: list[bytes] = []

    class FakeSocket:
        def __init__(self, *_a, **_kw): pass
        def setsockopt(self, *_a): pass
        def sendto(self, data, _addr): sent.append(data)
        def close(self): pass

    monkeypatch.setattr(socket, "socket", FakeSocket)
    assert mesh_wol.wake("aa:bb:cc:dd:ee:ff") is True
    # We send to UDP/9 AND UDP/7 so we expect two packets — both identical
    assert len(sent) == 2
    expected = b"\xff" * 6 + bytes.fromhex("aabbccddeeff") * 16
    assert sent[0] == expected == sent[1]
    assert len(expected) == 102
