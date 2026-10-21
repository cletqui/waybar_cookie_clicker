use clap::{Args, ValueEnum};
use serde_json::{Map, Value, json};

#[derive(Clone, Copy, ValueEnum)]
pub enum Direction {
    Right,
    Left,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Mode {
    Watch,
    Poll,
}

#[derive(Args)]
pub struct ConfigArgs {
    /// Side the slot panel expands towards, relative to the cookie button
    #[arg(long, value_enum, default_value = "right")]
    pub direction: Direction,

    /// Number of buy slots
    #[arg(long, default_value_t = 8)]
    pub slots: usize,

    /// watch: one long-running process per module, refreshed when the save changes.
    /// poll: waybar respawns the binary every interval and on SIGRTMIN+signal.
    #[arg(long, value_enum, default_value = "watch")]
    pub mode: Mode,

    /// Waybar signal number (SIGRTMIN+N) used for instant refresh (poll mode only)
    #[arg(long, default_value_t = 9)]
    pub signal: u8,

    /// Refresh interval in seconds for the cookie button
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub interval: u32,

    /// Refresh interval in seconds for the slots
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub slot_interval: u32,

    /// Drawer transition duration in milliseconds
    #[arg(long, default_value_t = 250)]
    pub transition_ms: u32,

    /// Reveal the panel on click instead of hover
    #[arg(long)]
    pub click_to_reveal: bool,

    /// Path of the launcher script as waybar should call it
    #[arg(long, default_value = "$HOME/.config/waybar/cookie-clicker/run.sh")]
    pub exec: String,
}

fn module(args: &ConfigArgs, sub: &str, every: u32, click: &str, middle: Option<&str>) -> Value {
    let run = &args.exec;
    let refresh = format!("pkill -SIGRTMIN+{} waybar", args.signal);
    let mut m = Map::new();
    m.insert("return-type".into(), json!("json"));
    match args.mode {
        Mode::Watch => {
            m.insert(
                "exec".into(),
                json!(format!("{run} --watch --every {every} {sub}")),
            );
            m.insert("on-click".into(), json!(format!("{run} {click}")));
            if let Some(mid) = middle {
                m.insert("on-click-middle".into(), json!(format!("{run} {mid}")));
            }
        }
        Mode::Poll => {
            m.insert("exec".into(), json!(format!("{run} {sub}")));
            m.insert("exec-on-event".into(), json!(true));
            m.insert("interval".into(), json!(every));
            m.insert("signal".into(), json!(args.signal));
            m.insert(
                "on-click".into(),
                json!(format!("{run} {click}; {refresh}")),
            );
            if let Some(mid) = middle {
                m.insert(
                    "on-click-middle".into(),
                    json!(format!("{run} {mid}; {refresh}")),
                );
            }
        }
    }
    Value::Object(m)
}

pub fn modules(args: &ConfigArgs) -> String {
    let mut out = Map::new();

    let mut members = vec![json!("custom/cookie-clicker")];
    members.extend((0..args.slots).map(|i| json!(format!("custom/cookie-s{i}"))));
    out.insert(
        "group/group-cookie".into(),
        json!({
            "orientation": "horizontal",
            "drawer": {
                "transition-duration": args.transition_ms,
                "transition-left-to-right": matches!(args.direction, Direction::Right),
                "click-to-reveal": args.click_to_reveal,
            },
            "modules": members,
        }),
    );

    out.insert(
        "custom/cookie-clicker".into(),
        module(args, "show", args.interval, "click", Some("reset")),
    );

    for i in 0..args.slots {
        let mut slot = module(
            args,
            &format!("slot {i}"),
            args.slot_interval,
            &format!("buy-slot {i}"),
            None,
        );
        slot["hide-empty-text"] = json!(true);
        out.insert(format!("custom/cookie-s{i}"), slot);
    }

    serde_json::to_string_pretty(&Value::Object(out)).unwrap_or_default() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> ConfigArgs {
        ConfigArgs {
            direction: Direction::Right,
            slots: 3,
            mode: Mode::Watch,
            signal: 9,
            interval: 1,
            slot_interval: 2,
            transition_ms: 250,
            click_to_reveal: false,
            exec: "/x/run.sh".into(),
        }
    }

    fn parse(a: &ConfigArgs) -> Value {
        serde_json::from_str(&modules(a)).unwrap()
    }

    #[test]
    fn right_direction_sets_left_to_right() {
        let v = parse(&args());
        assert_eq!(
            v["group/group-cookie"]["drawer"]["transition-left-to-right"],
            true
        );
    }

    #[test]
    fn left_direction_clears_left_to_right() {
        let mut a = args();
        a.direction = Direction::Left;
        let v = parse(&a);
        assert_eq!(
            v["group/group-cookie"]["drawer"]["transition-left-to-right"],
            false
        );
    }

    #[test]
    fn slot_count_drives_group_and_modules() {
        let mut a = args();
        a.slots = 5;
        let v = parse(&a);
        let members = v["group/group-cookie"]["modules"].as_array().unwrap();
        assert_eq!(members.len(), 6);
        assert_eq!(members[0], "custom/cookie-clicker");
        assert_eq!(members[5], "custom/cookie-s4");
        assert!(v.get("custom/cookie-s4").is_some());
        assert!(v.get("custom/cookie-s5").is_none());
    }

    #[test]
    fn zero_slots_leaves_only_the_button() {
        let mut a = args();
        a.slots = 0;
        let v = parse(&a);
        assert_eq!(
            v["group/group-cookie"]["modules"].as_array().unwrap().len(),
            1
        );
    }

    #[test]
    fn click_to_reveal_and_transition_are_passed_through() {
        let mut a = args();
        a.click_to_reveal = true;
        a.transition_ms = 500;
        let v = parse(&a);
        assert_eq!(v["group/group-cookie"]["drawer"]["click-to-reveal"], true);
        assert_eq!(
            v["group/group-cookie"]["drawer"]["transition-duration"],
            500
        );
    }

    #[test]
    fn watch_mode_uses_long_running_exec_without_polling_or_signals() {
        let v = parse(&args());
        let s1 = &v["custom/cookie-s1"];
        assert_eq!(s1["exec"], "/x/run.sh --watch --every 2 slot 1");
        assert_eq!(s1["on-click"], "/x/run.sh buy-slot 1");
        assert_eq!(s1["hide-empty-text"], true);
        for key in ["interval", "signal", "exec-on-event"] {
            assert!(s1.get(key).is_none(), "{key} must be absent in watch mode");
        }
        let b = &v["custom/cookie-clicker"];
        assert_eq!(b["exec"], "/x/run.sh --watch --every 1 show");
        assert_eq!(b["on-click-middle"], "/x/run.sh reset");
    }

    #[test]
    fn poll_mode_uses_interval_and_signal() {
        let mut a = args();
        a.mode = Mode::Poll;
        a.signal = 12;
        let v = parse(&a);
        let s0 = &v["custom/cookie-s0"];
        assert_eq!(s0["exec"], "/x/run.sh slot 0");
        assert_eq!(s0["interval"], 2);
        assert_eq!(s0["signal"], 12);
        assert_eq!(s0["exec-on-event"], true);
        assert_eq!(
            s0["on-click"],
            "/x/run.sh buy-slot 0; pkill -SIGRTMIN+12 waybar"
        );
        let b = &v["custom/cookie-clicker"];
        assert_eq!(
            b["on-click-middle"],
            "/x/run.sh reset; pkill -SIGRTMIN+12 waybar"
        );
    }

    #[test]
    fn slots_have_no_middle_click_reset() {
        let v = parse(&args());
        assert!(v["custom/cookie-s0"].get("on-click-middle").is_none());
    }
}
