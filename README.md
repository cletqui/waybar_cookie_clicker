# waybar-cookie-clicker

A Cookie Clicker game embedded in your [waybar](https://github.com/Alexays/Waybar) status bar.

Click the cookie button to earn cookies. Hover over it to reveal an expandable panel of buildings and upgrades. Passive income accumulates in the background — even across reboots and suspend.

---

## Features

- **8 buildings** — Cursor, Grandma, Farm, Mine, Factory, Bank, Temple, Wizard Tower — each buyable multiple times with 1.15× cost scaling
- **32 upgrades** — per-building doublers (up to 4 tiers each, every building included) and a 5-tier click power chain
- **Passive CPS** — income accumulates based on real elapsed time (unix timestamps, survives suspend, capped at 8 h offline)
- **Progressive reveal** — slots stay hidden until you can actually afford an item; no spoilers
- **Expandable slot panel** — hover the cookie button to reveal up to 8 slots; affordable items glow, empty slots collapse automatically
- **Instant refresh** — one long-running watcher per module picks up clicks and purchases within ~100 ms, with no process respawning
- **Save file in repo** — `state.json` lives next to the code for easy backup (gitignored, so clones start fresh)

---

## Requirements

- [Rust](https://rustup.rs/) (stable)
- [waybar](https://github.com/Alexays/Waybar) (developed and tested on 0.15)
- A [Nerd Font](https://www.nerdfonts.com/) (building icons use nerd glyphs)

---

## Installation

```bash
git clone https://github.com/youruser/waybar_cookie_clicker
cd waybar_cookie_clicker
cargo build --release
```

### Symlink into waybar config (recommended)

```bash
ln -s /path/to/waybar_cookie_clicker/waybar ~/.config/waybar/cookie-clicker
```

This makes `run.sh`, `modules.json`, and `style.css` available at a stable path. Updates to the repo are reflected immediately — no re-copying.

---

## Waybar integration

### 1. Add the CSS import

At the top of your `~/.config/waybar/style.css`:

```css
@import "cookie-clicker/style.css";
```

### 2. Include the module definitions

In your `~/.config/waybar/config.jsonc`, add the include path to each bar:

```jsonc
"include": [
  "~/.config/waybar/modules.json",
  "~/.config/waybar/cookie-clicker/modules.json"
]
```

### 3. Add the group to your bar

In `config.jsonc`, add `"group/group-cookie"` to `modules-right`:

```jsonc
"modules-right": [
  "group/group-cookie",
  "group/group-power"
]
```

### 4. Customise the layout (optional)

`waybar/modules.json` is generated. Regenerate it with the options you want:

```bash
./target/release/waybar_cookie_clicker config --direction left --slots 6 --signal 12 > waybar/modules.json
```

| Option              | Default                                      | Effect                                                         |
| ------------------- | -------------------------------------------- | -------------------------------------------------------------- |
| `--direction`       | `right`                                      | Side the panel expands towards (`right` / `left`)              |
| `--slots`           | `8`                                          | Number of buy slots                                            |
| `--mode`            | `watch`                                      | `watch`: one long-running process per module, no respawning; `poll`: waybar respawns the binary every interval |
| `--interval`        | `1`                                          | Button refresh interval in seconds                             |
| `--slot-interval`   | `1`                                          | Slot refresh interval in seconds                               |
| `--signal`          | `9`                                          | SIGRTMIN+N used for instant refresh (`poll` mode only)        |
| `--transition-ms`   | `250`                                        | Drawer animation duration                                      |
| `--click-to-reveal` | off                                          | Open the panel on click instead of hover                       |
| `--exec`            | `$HOME/.config/waybar/cookie-clicker/run.sh` | Launcher path as waybar should call it                         |

In `watch` mode each module runs `--watch` once: it re-reads the save when the file changes (so clicks show up within ~100 ms) and otherwise every `--interval` seconds, printing only when the output changed. Watchers exit when waybar goes away. No signals are involved.

Styling uses the classes `.cookie-button`, `.cookie-slot`, `.affordable`, `.locked` and `.has-upgrade`; override them in your own `style.css`.

### 5. Reload waybar

```bash
pkill -x waybar; waybar &
```

---

## Usage

| Action                    | How                                  |
| ------------------------- | ------------------------------------ |
| Earn a cookie             | Left-click the `󰆘` button            |
| See buildings / CPS       | Hover the `󰆘` button (tooltip)       |
| Open buy panel            | Hover the `󰆘` button (group expands) |
| Buy a building or upgrade | Left-click a slot in the panel       |
| Reset all progress        | Middle-click the `󰆘` button          |

---

## Game mechanics

### Buildings

Each building produces cookies per second (CPS). Buying more of the same building raises its cost by **1.15×** per unit owned.

| Building     | Base cost   | Base CPS |
| ------------ | ----------- | -------- |
| Cursor       | 15          | 0.1/s    |
| Grandma      | 100         | 0.5/s    |
| Farm         | 1,100       | 4/s      |
| Mine         | 12,000      | 10/s     |
| Factory      | 130,000     | 40/s     |
| Bank         | 1,400,000   | 100/s    |
| Temple       | 20,000,000  | 400/s    |
| Wizard Tower | 330,000,000 | 6,666/s  |

### Upgrades

One-time purchases that double a building's CPS or boost click power. They unlock automatically when their condition is met and only appear in the slot panel once you can afford them.

**Click chain** (each unlocks the next):
Reinforced clicking → Mouse wheel → Plastic mouse → Iron mouse → Titanium mouse

**Per-building chains**: 3–4 doublers per building, unlocking at 1 / 5 / 25 / 50 owned.

**Global multipliers**: unlock at 1 M and 100 M total cookies ever produced.

### Slot panel

Slots only reveal items you can actually buy right now (or buildings you already own). Everything else stays hidden — no seeing the endgame from the start.

Ordering: cheapest affordable items first, then owned buildings by next-purchase cost.

---

## CLI reference

```
waybar_cookie_clicker [--state <path>] [--watch [--every <secs>]] <command>

Commands:
  show             Output waybar JSON for the main cookie button
  click            Earn cookies_per_click cookies
  slot <N>         Output waybar JSON for slot N (0-indexed)
  config [opts]    Print waybar module definitions for a layout
  buy-slot <N>     Purchase the item in slot N
  reset            Wipe all progress and start over
```

`--state` defaults to `~/.local/share/waybar_cookie_clicker/state.json` when called directly. The `waybar/run.sh` wrapper always passes the repo's `state.json`.

---

## Adding buildings / upgrades

Buildings and upgrades are defined in `src/game/defs.rs`. See the doc comments on the definition structs there for the fields. After adding entries, run `cargo build --release` — no other files need changing.

---

## License

MIT

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```
