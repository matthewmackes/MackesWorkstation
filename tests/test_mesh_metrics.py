"""mesh_metrics — Prometheus exporter output parsing."""
from __future__ import annotations


SAMPLE_METRICS = """\
# HELP wireguard_sent_bytes_total Sent bytes per peer
# TYPE wireguard_sent_bytes_total counter
wireguard_sent_bytes_total{interface="wg0",public_key="abc123def456",friendly_name="alpha"} 12345
wireguard_sent_bytes_total{interface="wg0",public_key="zzz",friendly_name="beta"} 67890
# HELP wireguard_received_bytes_total Received bytes per peer
# TYPE wireguard_received_bytes_total counter
wireguard_received_bytes_total{interface="wg0",public_key="abc123def456",friendly_name="alpha"} 999
"""


def test_parse_metrics_groups_by_friendly_name(monkeypatch):
    from mackes import mesh_metrics
    monkeypatch.setattr(mesh_metrics, "exporter_metrics",
                        lambda timeout=3.0: SAMPLE_METRICS)
    parsed = mesh_metrics.parsed_per_peer_metrics()
    assert "alpha" in parsed
    assert "beta" in parsed
    assert parsed["alpha"]["wireguard_sent_bytes_total"] == 12345
    assert parsed["alpha"]["wireguard_received_bytes_total"] == 999
    assert parsed["beta"]["wireguard_sent_bytes_total"] == 67890


def test_parse_metrics_handles_empty_response(monkeypatch):
    from mackes import mesh_metrics
    monkeypatch.setattr(mesh_metrics, "exporter_metrics",
                        lambda timeout=3.0: None)
    assert mesh_metrics.parsed_per_peer_metrics() == {}


def test_parse_metrics_uses_public_key_prefix_when_no_friendly_name(monkeypatch):
    from mackes import mesh_metrics
    text = ('wireguard_sent_bytes_total{public_key="0123456789abcdef"} 42\n')
    monkeypatch.setattr(mesh_metrics, "exporter_metrics",
                        lambda timeout=3.0: text)
    parsed = mesh_metrics.parsed_per_peer_metrics()
    # Falls back to first 12 chars of public_key
    assert "0123456789ab" in parsed
    assert parsed["0123456789ab"]["wireguard_sent_bytes_total"] == 42
