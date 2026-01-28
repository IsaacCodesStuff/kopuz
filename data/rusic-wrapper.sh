#!/bin/bash
# Wrapper script to ensure display environment is set for WebKit subprocesses

# Export display variables to ensure WebKit processes inherit them
export DISPLAY="${DISPLAY:-:0}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"

# X11 authentication - find XAUTHORITY (including Mutter XWayland auth files)
if [ -z "$XAUTHORITY" ]; then
    # Check for Mutter XWayland auth file (used on GNOME Wayland)
    MUTTER_AUTH=$(find "$XDG_RUNTIME_DIR" -name '.mutter-Xwaylandauth.*' 2>/dev/null | head -1)
    if [ -n "$MUTTER_AUTH" ] && [ -f "$MUTTER_AUTH" ]; then
        export XAUTHORITY="$MUTTER_AUTH"
    elif [ -f "$HOME/.Xauthority" ]; then
        export XAUTHORITY="$HOME/.Xauthority"
    fi
fi

# Run the actual application
exec /app/bin/rusic "$@"
