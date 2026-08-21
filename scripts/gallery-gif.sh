#!/usr/bin/env bash
# Record the gallery tour into assets/gallery.gif and book/src/gallery.gif.
# Needs a display, ffmpeg, xwininfo, wmctrl, import, and python3 (XTest).
# Default: Xephyr + metacity so the live desktop cannot appear in a frame.
# ICEDTEA_GALLERY_ISOLATED=0 records on the current display (float a tiler first).
set -eu
# pipefail is off: xdpyinfo | awk-exit and identify | awk-exit are SIGPIPE.

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [[ -z "${DISPLAY:-}" ]]; then
  echo "gallery-gif: DISPLAY is not set" >&2
  exit 1
fi
for cmd in ffmpeg xwininfo wmctrl import python3 xprop xdotool; do
  if ! command -v "$cmd" >/dev/null; then
    echo "gallery-gif: missing $cmd" >&2
    exit 1
  fi
done

# Isolated X server: no host windows, no host gsettings, no mosaic Super+g.
if [[ "${ICEDTEA_GALLERY_ISOLATED:-1}" != "0" && -z "${ICEDTEA_GALLERY_NESTED:-}" ]]; then
  if ! command -v Xephyr >/dev/null; then
    echo "gallery-gif: Xephyr is missing; install xserver-xephyr or set ICEDTEA_GALLERY_ISOLATED=0" >&2
    exit 1
  fi
  if ! command -v metacity >/dev/null; then
    echo "gallery-gif: metacity is missing; install metacity or set ICEDTEA_GALLERY_ISOLATED=0" >&2
    exit 1
  fi
  if ! command -v dbus-run-session >/dev/null; then
    echo "gallery-gif: dbus-run-session is missing" >&2
    exit 1
  fi
  display_n=""
  for n in $(seq 3 20); do
    if [[ ! -e /tmp/.X11-unix/X$n ]]; then
      display_n=$n
      break
    fi
  done
  if [[ -z "$display_n" ]]; then
    echo "gallery-gif: no free X display in :3-:20" >&2
    exit 1
  fi
  # Client 1600x900 plus a title bar and a place origin.
  Xephyr ":$display_n" -screen 1720x1080 -ac -nolisten tcp \
    -title icedtea-gallery-record +extension RANDR +extension COMPOSITE +extension XTEST &
  xephyr_pid=$!
  isolated_cleanup() {
    kill "$xephyr_pid" 2>/dev/null || true
    wait "$xephyr_pid" 2>/dev/null || true
  }
  trap isolated_cleanup EXIT
  for _ in $(seq 1 50); do
    if [[ -e /tmp/.X11-unix/X$display_n ]]; then
      break
    fi
    if ! kill -0 "$xephyr_pid" 2>/dev/null; then
      echo "gallery-gif: Xephyr exited before the display came up" >&2
      exit 1
    fi
    sleep 0.1
  done
  if [[ ! -e /tmp/.X11-unix/X$display_n ]]; then
    echo "gallery-gif: Xephyr display :$display_n did not appear" >&2
    exit 1
  fi
  echo "gallery-gif: recording inside Xephyr :$display_n"
  export DISPLAY=":$display_n"
  unset WAYLAND_DISPLAY
  export ICEDTEA_GALLERY_NESTED=1
  export ICEDTEA_GALLERY_ISOLATED=1
  set +e
  dbus-run-session -- bash "$0"
  status=$?
  set -e
  isolated_cleanup
  trap - EXIT
  exit "$status"
fi

if [[ -n "${ICEDTEA_GALLERY_NESTED:-}" ]]; then
  metacity --sm-disable &
  sleep 0.4
  if command -v xsetroot >/dev/null; then
    xsetroot -solid '#1a1a1a' || true
  fi
fi

# Must match Boot::size in the gallery tour path.
client_w=1600
client_h=900
min_w=1400
min_h=800
hold_ms=2000
dest="$root/assets/gallery.gif"
book="$root/book/src/gallery.gif"

screen="$(xdpyinfo | awk '/dimensions:/ { print $2; exit }')"
screen_w="${screen%x*}"
screen_h="${screen#*x}"
work="$(xprop -root _NET_WORKAREA 2>/dev/null | awk -F'[ ,]' '{
  for (i = 1; i <= NF; i++) {
    if ($i ~ /^[0-9]+$/) {
      print $i
      n++
      if (n == 4) exit
    }
  }
}' | tr '\n' ' ')"
read -r work_x work_y work_w work_h <<<"$work"
if [[ -z "${work_w:-}" || "$work_w" -lt 100 ]]; then
  work_x=0
  work_y=0
  work_w=$screen_w
  work_h=$screen_h
fi
origin_x=$((work_x + 40))
origin_y=$((work_y + 48))
if [[ -z "${ICEDTEA_GALLERY_NESTED:-}" ]]; then
  origin_y=$((work_y + 8))
fi
if (( origin_x + client_w > work_x + work_w )); then
  origin_x=$work_x
fi
if (( origin_y + client_h > work_y + work_h )); then
  origin_y=$work_y
fi

wm_name="$(wmctrl -m 2>/dev/null | awk -F: '/^Name:/ { print $2; exit }' | xargs || true)"
tiler="none"
case "$(printf '%s' "$wm_name" | tr '[:upper:]' '[:lower:]')" in
  *i3*) tiler=i3 ;;
  *sway*) tiler=sway ;;
  *bspwm*) tiler=bspwm ;;
  *awesome*) tiler=awesome ;;
  *xmonad*) tiler=xmonad ;;
  *herbst*) tiler=herbstluft ;;
esac
enabled_ext=""
if [[ -z "${ICEDTEA_GALLERY_NESTED:-}" ]] && command -v gnome-extensions >/dev/null; then
  enabled_ext="$(gnome-extensions list --enabled 2>/dev/null || true)"
fi
if grep -qi 'mosaic' <<<"$enabled_ext"; then
  tiler=mosaic
elif grep -qiE 'paperwm|forge|pop-shell|tiling-assistant|gtile' <<<"$enabled_ext"; then
  tiler=gnome-tile
fi

echo "gallery-gif: wm='${wm_name:-unknown}' tiler=$tiler screen=${screen_w}x${screen_h} work=${work_w}x${work_h}+${work_x}+${work_y}"

send_hotkey() {
  local spec="$1"
  python3 - "$spec" <<'PY'
import ctypes
import sys
import time

spec = sys.argv[1]
x11 = ctypes.cdll.LoadLibrary("libX11.so.6")
xtst = ctypes.cdll.LoadLibrary("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
x11.XKeysymToKeycode.restype = ctypes.c_uint
x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XFlush.argtypes = [ctypes.c_void_p]
x11.XStringToKeysym.restype = ctypes.c_ulong
x11.XStringToKeysym.argtypes = [ctypes.c_char_p]
xtst.XTestFakeKeyEvent.argtypes = [
    ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong
]
dpy = x11.XOpenDisplay(None)
if not dpy:
    sys.exit("XOpenDisplay failed")
mod_names = {"super": b"Super_L", "shift": b"Shift_L", "ctrl": b"Control_L", "alt": b"Alt_L"}
parts = spec.lower().replace("<", "").replace(">", " ").replace("+", " ").split()
keys = []
for p in parts:
    if p in ("primary", "control"):
        p = "ctrl"
    if p in mod_names:
        sym = x11.XStringToKeysym(mod_names[p])
    else:
        sym = x11.XStringToKeysym(p.encode())
        if not sym:
            sym = x11.XStringToKeysym(p[:1].upper().encode() + p[1:].encode())
    if not sym:
        sys.exit(f"unknown key {p}")
    keys.append(sym)
codes = []
for sym in keys:
    code = x11.XKeysymToKeycode(dpy, sym)
    if code == 0:
        sys.exit(f"no keycode for {hex(sym)}")
    codes.append(code)
for code in codes:
    xtst.XTestFakeKeyEvent(dpy, code, 1, 0)
x11.XFlush(dpy)
time.sleep(0.04)
for code in reversed(codes):
    xtst.XTestFakeKeyEvent(dpy, code, 0, 0)
x11.XFlush(dpy)
time.sleep(0.05)
PY
}

window_geom() {
  local id="$1"
  local info
  info="$(xwininfo -id "$id")"
  awk '
    /Absolute upper-left X/ { x = $NF }
    /Absolute upper-left Y/ { y = $NF }
    /^  Width:/ { w = $NF }
    /^  Height:/ { h = $NF }
    /Map State:/ { map = $NF }
    END { print x+0, y+0, w+0, h+0, map }
  ' <<<"$info"
}

on_screen() {
  local x="$1" y="$2" w="$3" h="$4"
  (( w >= min_w && h >= min_h )) || return 1
  (( x >= 0 && y >= 0 )) || return 1
  (( x + w <= screen_w + 4 && y + h <= screen_h + 4 )) || return 1
  return 0
}

place_window() {
  local id="$1"
  wmctrl -i -r "$id" -b remove,maximized_vert,maximized_horz,fullscreen,hidden || true
  wmctrl -i -r "$id" -e "0,${origin_x},${origin_y},${client_w},${client_h}" || true
  wmctrl -i -a "$id" || true
  wmctrl -i -r "$id" -b add,above || true
}

float_window() {
  local id="$1"
  local dec=$((id))
  case "$tiler" in
    i3)
      i3-msg "[id=${dec}] floating enable, sticky disable, border normal" >/dev/null
      ;;
    sway)
      swaymsg "[id=${dec}] floating enable" >/dev/null
      ;;
    bspwm)
      bspc node "$id" -t floating || true
      ;;
    mosaic)
      wmctrl -i -a "$id" || true
      local bind
      bind="$(gsettings --schemadir "$HOME/.local/share/gnome-shell/extensions/gnome-mosaic@jardon.github.com/schemas" \
        get org.gnome.shell.extensions.gnome-mosaic toggle-floating 2>/dev/null || true)"
      bind="${bind#*[\'\"]}"
      bind="${bind%%[\'\"]*}"
      bind="${bind:-<Super>g}"
      send_hotkey "$bind"
      ;;
    gnome-tile)
      wmctrl -i -a "$id" || true
      send_hotkey "Super+Down"
      ;;
    *)
      return 0
      ;;
  esac
}

# Mosaic reports a fake 1600x900 at 40,40 while the window is still tiled.
# Always float first when a tiler is active, then place, then trust geometry.
ensure_placed() {
  local id="$1"
  local i x y w h map
  if [[ "$tiler" != "none" ]]; then
    echo "gallery-gif: floating under tiler=$tiler (xwininfo is not the compositor)"
    float_window "$id"
    sleep 0.35
  fi
  for i in 1 2 3 4 5 6 7 8; do
    place_window "$id"
    sleep 0.2
    read -r x y w h map <<<"$(window_geom "$id")"
    if [[ "$map" == "IsViewable" ]] && on_screen "$x" "$y" "$w" "$h"; then
      echo "gallery-gif: placed ${w}x${h}+${x}+${y}"
      return 0
    fi
    echo "gallery-gif: retry place (now ${w}x${h}+${x}+${y})"
    if [[ "$tiler" != "none" ]]; then
      float_window "$id"
      sleep 0.25
    fi
  done
  echo "gallery-gif: could not place a ${client_w}x${client_h} window on ${screen_w}x${screen_h} (last ${w}x${h}+${x}+${y}, tiler=$tiler)" >&2
  return 1
}

# Mutter SSD follows the session gtk-theme / color-scheme, not iced Theme::custom.
# Existing windows keep the mapped decoration until they are remapped.
saved_scheme=""
saved_gtk_theme=""
last_host_face=""
if [[ -z "${ICEDTEA_GALLERY_NESTED:-}" ]] && command -v gsettings >/dev/null; then
  saved_scheme="$(gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null || true)"
  saved_gtk_theme="$(gsettings get org.gnome.desktop.interface gtk-theme 2>/dev/null || true)"
fi

remap_window() {
  local id="$1"
  python3 - "$id" <<'PY'
import ctypes
import sys
import time

wid = int(sys.argv[1], 16)
x11 = ctypes.cdll.LoadLibrary("libX11.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
x11.XUnmapWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XMapWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XFlush.argtypes = [ctypes.c_void_p]
dpy = x11.XOpenDisplay(None)
if not dpy:
    sys.exit("XOpenDisplay failed")
x11.XUnmapWindow(dpy, wid)
x11.XFlush(dpy)
time.sleep(0.2)
x11.XMapWindow(dpy, wid)
x11.XFlush(dpy)
PY
}

apply_host_chrome() {
  local id="$1"
  local face="dark"
  if [[ -f "${ackfile}.face" ]]; then
    face="$(tr -d '[:space:]' <"${ackfile}.face")"
  fi
  if [[ -z "${ICEDTEA_GALLERY_NESTED:-}" ]] && command -v gsettings >/dev/null; then
    local theme="${saved_gtk_theme//\'/}"
    if [[ "$face" == "light" ]]; then
      gsettings set org.gnome.desktop.interface color-scheme prefer-light || true
      gsettings set org.gnome.desktop.interface gtk-theme "${theme%-dark}" || true
    else
      gsettings set org.gnome.desktop.interface color-scheme prefer-dark || true
      if [[ -n "$theme" ]]; then
        gsettings set org.gnome.desktop.interface gtk-theme "$theme" || true
      fi
    fi
  fi
  xprop -id "$id" -f _GTK_THEME_VARIANT 8u -set _GTK_THEME_VARIANT "$face" || true
  if [[ "$face" != "$last_host_face" ]]; then
    # Mutter reads gtk-theme at map time; give the setting a beat to land.
    sleep 0.45
    remap_window "$id"
    ensure_placed "$id"
    last_host_face="$face"
  fi
  wmctrl -i -a "$id" || true
  echo "gallery-gif: host chrome face=$face"
}

# Visible frame rectangle (client plus window-manager chrome).
frame_geom() {
  local id="$1"
  local x y w h map ext L rest R T B
  read -r x y w h map <<<"$(window_geom "$id")"
  if ! on_screen "$x" "$y" "$w" "$h"; then
    echo "gallery-gif: window moved off-screen (${w}x${h}+${x}+${y})" >&2
    return 1
  fi
  ext="$(xprop -id "$id" _NET_FRAME_EXTENTS 2>/dev/null | awk -F'= ' '{ print $2 }' | tr -d ' ')"
  L=0
  R=0
  T=0
  B=0
  if [[ "$ext" == *,*,*,* ]]; then
    L="${ext%%,*}"; rest="${ext#*,}"
    R="${rest%%,*}"; rest="${rest#*,}"
    T="${rest%%,*}"; B="${rest#*,}"
  fi
  fx=$((x - L))
  fy=$((y - T))
  fw=$((w + L + R))
  fh=$((h + T + B))
  if (( fx < 0 || fy < 0 || fx + fw > screen_w + 4 || fy + fh > screen_h + 4 )); then
    echo "gallery-gif: frame ${fw}x${fh}+${fx}+${fy} is not on screen" >&2
    return 1
  fi
}

# import -window is the visible backing store (cropped by the tile).
# Crop the root at the frame rectangle after the window is floating.
capture_window() {
  local id="$1"
  local dest="$2"
  local x y w h map ext L rest R T B fx fy fw fh
  read -r x y w h map <<<"$(window_geom "$id")"
  if ! on_screen "$x" "$y" "$w" "$h"; then
    echo "gallery-gif: window moved off-screen before capture (${w}x${h}+${x}+${y})" >&2
    return 1
  fi
  ext="$(xprop -id "$id" _NET_FRAME_EXTENTS 2>/dev/null | awk -F'= ' '{ print $2 }' | tr -d ' ')"
  L=0
  R=0
  T=0
  B=0
  if [[ "$ext" == *,*,*,* ]]; then
    L="${ext%%,*}"; rest="${ext#*,}"
    R="${rest%%,*}"; rest="${rest#*,}"
    T="${rest%%,*}"; B="${rest#*,}"
  fi
  fx=$((x - L))
  fy=$((y - T))
  fw=$((w + L + R))
  fh=$((h + T + B))
  if (( fx < 0 || fy < 0 || fx + fw > screen_w + 4 || fy + fh > screen_h + 4 )); then
    echo "gallery-gif: frame ${fw}x${fh}+${fx}+${fy} is not on screen" >&2
    return 1
  fi
  import -window root -crop "${fw}x${fh}+${fx}+${fy}" +repage "$dest"
}

png_ok() {
  local path="$1"
  local wh w h
  wh="$(identify -format '%wx%h' "$path")"
  w="${wh%x*}"
  h="${wh#*x}"
  if (( w < min_w || h < min_h )); then
    echo "gallery-gif: capture ${wh} is below ${min_w}x${min_h} ($path)" >&2
    return 1
  fi
  return 0
}

cargo build -p icedtea-gallery
bin="$root/target/debug/icedtea-gallery"

pkill -x icedtea-gallery 2>/dev/null || true
sleep 0.3

workdir="$(mktemp -d)"
cleanup() {
  if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  fi
  if command -v gsettings >/dev/null; then
    if [[ -n "${saved_scheme:-}" ]]; then
      gsettings set org.gnome.desktop.interface color-scheme "${saved_scheme//\'/}" || true
    fi
    if [[ -n "${saved_gtk_theme:-}" ]]; then
      gsettings set org.gnome.desktop.interface gtk-theme "${saved_gtk_theme//\'/}" || true
    fi
  fi
  rm -rf "$workdir"
}
trap cleanup EXIT

lenfile="$workdir/tour_len"
cmdfile="$workdir/cmd"
ackfile="$workdir/ack"
inject="$workdir/inject"
printf '0\n' >"$cmdfile"
: >"$inject"

ICEDTEA_GALLERY_TOUR=1 \
  ICEDTEA_GALLERY_TOUR_LEN_FILE="$lenfile" \
  ICEDTEA_GALLERY_TOUR_CMD="$cmdfile" \
  ICEDTEA_GALLERY_TOUR_ACK="$ackfile" \
  ICEDTEA_GALLERY_INJECT="$inject" \
  "$bin" &
pid=$!

pages=""
for _ in $(seq 1 80); do
  if [[ -f "$lenfile" ]]; then
    pages="$(tr -d '[:space:]' <"$lenfile")"
    if [[ "$pages" =~ ^[0-9]+$ ]] && (( pages > 1 )); then
      break
    fi
  fi
  pages=""
  sleep 0.1
done
if [[ -z "$pages" ]]; then
  echo "gallery-gif: gallery did not report the tour length" >&2
  exit 1
fi

wid=""
for _ in $(seq 1 80); do
  wid="$(wmctrl -lp | awk -v p="$pid" '$3 == p { print $1; exit }')"
  if [[ -n "$wid" ]]; then
    break
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "gallery-gif: gallery exited before the window appeared" >&2
    exit 1
  fi
  sleep 0.25
done
if [[ -z "$wid" ]]; then
  echo "gallery-gif: timed out waiting for the window" >&2
  exit 1
fi

ensure_placed "$wid"

ack=""
for _ in $(seq 1 80); do
  if [[ -f "$ackfile" ]]; then
    ack="$(tr -d '[:space:]' <"$ackfile")"
    if [[ "$ack" == "0" ]]; then
      break
    fi
  fi
  ack=""
  sleep 0.05
done
if [[ "$ack" != "0" ]]; then
  echo "gallery-gif: gallery did not acknowledge beat 0" >&2
  exit 1
fi

sleep 0.4
capture_window "$wid" "$workdir/probe.png"
if ! png_ok "$workdir/probe.png"; then
  echo "gallery-gif: probe grab is cropped; window is not fully visible" >&2
  exit 1
fi
echo "gallery-gif: probe $(identify -format '%wx%h' "$workdir/probe.png")"

caption_name=""
caption_dir=""
for query in \
  "FiraCode Nerd Font Propo:style=Bold" \
  "FiraCode Nerd Font:style=Bold" \
  "FuraCode Nerd Font:style=Bold" \
  "Fira Sans:style=Bold"; do
  file="$(fc-match -f '%{file}' "$query" 2>/dev/null || true)"
  family="$(fc-match -f '%{family}' "$query" 2>/dev/null || true)"
  family="${family%%,*}"
  case "$family" in
    Fira*|Fura*)
      if [[ -n "$file" && -f "$file" ]]; then
        caption_name=$family
        caption_dir="$(dirname "$file")"
        break
      fi
      ;;
  esac
done
if [[ -z "$caption_name" || -z "$caption_dir" ]]; then
  echo "gallery-gif: need Fira or Fura (fc-match)" >&2
  exit 1
fi
echo "gallery-gif: caption font=$caption_name dir=$caption_dir"

frame_geom "$wid" || exit 1
# libx264 needs even dimensions.
if (( fw % 2 )); then fw=$((fw - 1)); fi
if (( fh % 2 )); then fh=$((fh - 1)); fi
disp="${DISPLAY#:}"
echo "gallery-gif: live grab ${fw}x${fh}+${fx},${fy} on :$disp"

# Continuous grab of the running window. Not a still sequence.
ffmpeg -y -hide_banner -loglevel error \
  -f x11grab -draw_mouse 1 -framerate 12 \
  -video_size "${fw}x${fh}" -i "${DISPLAY}+${fx},${fy}" \
  -c:v libx264 -preset ultrafast -pix_fmt yuv420p \
  "$workdir/live.mkv" &
grab_pid=$!
sleep 0.4
if ! kill -0 "$grab_pid" 2>/dev/null; then
  echo "gallery-gif: ffmpeg x11grab failed to start" >&2
  exit 1
fi
t0="$(date +%s.%N)"

# Directed pointer demo. Catalog stills stay on `just gallery-qa`.
if ! python3 "$root/scripts/gallery_qa.py" --record-demo \
  --wid "$wid" \
  --cmd "$cmdfile" \
  --ack "$ackfile" \
  --srt "$workdir/captions.srt" \
  --t0 "$t0" \
  --inject "$inject"; then
  echo "gallery-gif: demo script failed" >&2
  kill "$grab_pid" 2>/dev/null || true
  exit 1
fi

sleep 0.25
kill -INT "$grab_pid" 2>/dev/null || true
wait "$grab_pid" 2>/dev/null || true

if [[ ! -s "$workdir/live.mkv" ]]; then
  echo "gallery-gif: live grab is empty" >&2
  exit 1
fi

# Encode the live grab (not %d.png stills) and burn step captions.
ffmpeg -y -hide_banner -loglevel error \
  -i "$workdir/live.mkv" \
  -vf "subtitles=$workdir/captions.srt:fontsdir=${caption_dir}:force_style='FontName=${caption_name},FontSize=32,Bold=1,PlayResY=${fh},PrimaryColour=&H00B2DBEB,OutlineColour=&H0021201D,BorderStyle=1,Outline=2,Shadow=1,MarginV=24',split[s0][s1];[s0]palettegen=max_colors=192:stats_mode=single[p];[s1][p]paletteuse=dither=sierra2_4a" \
  -loop 0 "$dest"
cp -f "$dest" "$book"

gif_wh="$(identify -format '%wx%h' "${dest}[0]")"
gif_w="${gif_wh%x*}"
gif_h="${gif_wh#*x}"
if (( gif_w < min_w || gif_h < min_h )); then
  echo "gallery-gif: wrote ${gif_wh}, below ${min_w}x${min_h}" >&2
  exit 1
fi

echo "gallery-gif: wrote $dest and $book (${gif_wh}, $(wc -c <"$dest") bytes, pointer demo, tiler=$tiler)"
