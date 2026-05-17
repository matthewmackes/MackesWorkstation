"""mesh_perf — tweak storage, MTU helpers, kernel-mode detection."""
from __future__ import annotations


def test_tailscale_up_flags_kernel_mode_when_available(isolated_xdg, monkeypatch):
    """When the user prefers kernel mode AND the wireguard module is
    available, tailscale_up_flags returns --tun + --netstack=false."""
    import importlib
    import mackes.mesh_perf
    importlib.reload(mackes.mesh_perf)
    mp = mackes.mesh_perf
    monkeypatch.setattr(mp, "kernel_mode_available", lambda: True)
    mp.set_use_kernel_mode(True)
    mp.set_preferred_mtu(0)
    flags = mp.tailscale_up_flags()
    assert "--tun=mackes-wg0" in flags
    assert "--netstack=false" in flags
    assert not any(f.startswith("--mtu=") for f in flags)


def test_tailscale_up_flags_mtu_when_lan_optimised(isolated_xdg, monkeypatch):
    import importlib
    import mackes.mesh_perf
    importlib.reload(mackes.mesh_perf)
    mp = mackes.mesh_perf
    monkeypatch.setattr(mp, "kernel_mode_available", lambda: False)
    mp.set_use_kernel_mode(False)
    mp.set_preferred_mtu(mp.LAN_MTU)
    flags = mp.tailscale_up_flags()
    assert any(f == f"--mtu={mp.LAN_MTU}" for f in flags)
    assert "--tun=mackes-wg0" not in flags


def test_tailscale_up_flags_off_when_all_disabled(isolated_xdg, monkeypatch):
    import importlib
    import mackes.mesh_perf
    importlib.reload(mackes.mesh_perf)
    mp = mackes.mesh_perf
    monkeypatch.setattr(mp, "kernel_mode_available", lambda: False)
    mp.set_use_kernel_mode(False)
    mp.set_preferred_mtu(0)
    assert mp.tailscale_up_flags() == []


def test_set_preferred_mtu_round_trip(isolated_xdg, monkeypatch):
    import importlib
    import mackes.mesh_perf
    importlib.reload(mackes.mesh_perf)
    mp = mackes.mesh_perf
    mp.set_preferred_mtu(mp.LAN_MTU)
    assert mp.preferred_mtu() == mp.LAN_MTU
    mp.set_preferred_mtu(0)
    assert mp.preferred_mtu() == 0


def test_kernel_mode_default_is_on(isolated_xdg):
    """Even with no tweaks.json, the kernel-mode preference should be
    True (kernel mode is strictly better when available)."""
    import importlib
    import mackes.mesh_perf
    importlib.reload(mackes.mesh_perf)
    assert mackes.mesh_perf.use_kernel_mode_preference() is True
