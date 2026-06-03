# Wallpapers

Shipped wallpapers ship into `/usr/share/mackes-shell/wallpapers/` via the
RPM. The shipped preset (`chupre.yaml`) references `chupre.jpg` here.

For development, drop the file named:

- `chupre.jpg`

If the referenced wallpaper is missing, `apply_appearance` skips it silently —
the rest of the preset still applies. Users can always set their own via
**Look & Feel → Appearance → Wallpaper**.
