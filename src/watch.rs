use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use std::{fs, thread};

use crate::game::engine;
use crate::game::state::GameState;
use crate::storage;

const POLL: Duration = Duration::from_millis(100);

pub fn needs_refresh(
    seen: Option<SystemTime>,
    current: Option<SystemTime>,
    since: Duration,
    every: Duration,
) -> bool {
    seen != current || since >= every
}

#[derive(Default)]
pub struct Watcher {
    mtime: Option<SystemTime>,
    refreshed: Option<Instant>,
    last: Option<String>,
}

impl Watcher {
    pub fn step<W: Write>(
        &mut self,
        path: &Path,
        now: Instant,
        every: Duration,
        render: &dyn Fn(&GameState) -> String,
        out: &mut W,
    ) -> io::Result<()> {
        let mtime = fs::metadata(path).and_then(|m| m.modified()).ok();
        let due = match self.refreshed {
            None => true,
            Some(at) => needs_refresh(self.mtime, mtime, now.duration_since(at), every),
        };
        if !due {
            return Ok(());
        }
        self.mtime = mtime;
        self.refreshed = Some(now);

        let mut state = storage::load(path);
        engine::tick(&mut state);
        let text = render(&state);
        if self.last.as_deref() != Some(text.as_str()) {
            writeln!(out, "{text}")?;
            out.flush()?;
            self.last = Some(text);
        }
        Ok(())
    }
}

fn stdout_closed() -> bool {
    let mut fd = libc::pollfd {
        fd: libc::STDOUT_FILENO,
        events: 0,
        revents: 0,
    };
    let ready = unsafe { libc::poll(&mut fd, 1, 0) };
    ready > 0 && fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0
}

pub fn run(path: &Path, every: Duration, render: &dyn Fn(&GameState) -> String) -> io::Result<()> {
    let parent = std::os::unix::process::parent_id();
    let mut watcher = Watcher::default();
    let mut out = io::stdout().lock();
    while std::os::unix::process::parent_id() == parent && !stdout_closed() {
        watcher.step(path, Instant::now(), every, render, &mut out)?;
        thread::sleep(POLL);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    fn t(secs: u64) -> Option<SystemTime> {
        Some(UNIX_EPOCH + Duration::from_secs(secs))
    }

    fn render(state: &GameState) -> String {
        format!("{}", state.cookies)
    }

    fn write_state(path: &Path, cookies: f64) {
        let state = GameState {
            cookies,
            ..Default::default()
        };
        storage::save(path, &state);
    }

    #[test]
    fn refresh_when_mtime_changed() {
        assert!(needs_refresh(
            t(1),
            t(2),
            Duration::ZERO,
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn refresh_when_interval_elapsed() {
        assert!(needs_refresh(
            t(1),
            t(1),
            Duration::from_secs(1),
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn no_refresh_when_idle() {
        assert!(!needs_refresh(
            t(1),
            t(1),
            Duration::from_millis(500),
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn first_step_prints() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        write_state(&path, 7.0);
        let mut out = Vec::new();
        Watcher::default()
            .step(
                &path,
                Instant::now(),
                Duration::from_secs(1),
                &render,
                &mut out,
            )
            .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "7\n");
    }

    #[test]
    fn unchanged_output_is_not_reprinted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        write_state(&path, 7.0);
        let mut out = Vec::new();
        let mut w = Watcher::default();
        let start = Instant::now();
        let every = Duration::from_secs(1);
        w.step(&path, start, every, &render, &mut out).unwrap();
        w.step(
            &path,
            start + Duration::from_secs(2),
            every,
            &render,
            &mut out,
        )
        .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "7\n");
    }

    #[test]
    fn changed_state_file_reprints_before_interval() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        write_state(&path, 7.0);
        let mut out = Vec::new();
        let mut w = Watcher::default();
        let start = Instant::now();
        let every = Duration::from_secs(60);
        w.step(&path, start, every, &render, &mut out).unwrap();
        std::thread::sleep(Duration::from_millis(20));
        write_state(&path, 9.0);
        w.step(
            &path,
            start + Duration::from_millis(100),
            every,
            &render,
            &mut out,
        )
        .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "7\n9\n");
    }

    #[test]
    fn idle_step_before_interval_does_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        write_state(&path, 7.0);
        let mut w = Watcher::default();
        let start = Instant::now();
        let every = Duration::from_secs(60);
        let mut out = Vec::new();
        w.step(&path, start, every, &render, &mut out).unwrap();
        let calls = std::cell::Cell::new(0);
        let counting = |s: &GameState| {
            calls.set(calls.get() + 1);
            render(s)
        };
        w.step(
            &path,
            start + Duration::from_millis(100),
            every,
            &counting,
            &mut out,
        )
        .unwrap();
        assert_eq!(calls.get(), 0);
    }

    #[test]
    fn write_error_propagates() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.json");
        let r = Watcher::default().step(
            &path,
            Instant::now(),
            Duration::from_secs(1),
            &render,
            &mut Broken,
        );
        assert!(r.is_err());
    }
}
