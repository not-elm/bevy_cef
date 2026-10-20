//! Process-level tests for webview profiles.
//!
//! CEF can be initialized only once per process, so the orchestrator re-runs this binary as role
//! apps against one root cache directory and checks what each role's test page reports. The tests
//! run only with `BEVY_CEF_PROCESS_TESTS=1`; macOS is verified by hand.

use serde::{Deserialize, Serialize};
use std::process::ExitCode;

/// Enables the tests; without `=1` the binary prints `skipped`.
const PROCESS_TESTS: &str = "BEVY_CEF_PROCESS_TESTS";
/// The role a child process runs; the orchestrator runs when it is unset.
const ROLE: &str = "BEVY_CEF_TEST_ROLE";
/// The root cache directory every role shares.
const ROOT: &str = "BEVY_CEF_TEST_ROOT";
/// The port of the orchestrator's HTTP server.
const PORT: &str = "BEVY_CEF_TEST_PORT";
/// The profile of the role's webviews; unset means no `WebviewProfile`.
const PROFILE: &str = "BEVY_CEF_TEST_PROFILE";
/// The value a `set` page stores in its cookies, localStorage and IndexedDB.
const TOKEN: &str = "BEVY_CEF_TEST_TOKEN";
/// `CefPlugin::cookie_flush_interval` in seconds; defaults to 1.
const FLUSH_SECS: &str = "BEVY_CEF_TEST_FLUSH_SECS";
/// The `ZoomLevel` a page webview is spawned with; defaults to 0.
const ZOOM: &str = "BEVY_CEF_TEST_ZOOM";
/// When set, a page role asks for a second report 500 ms after `LoadFinished`.
const REREPORT: &str = "BEVY_CEF_TEST_REREPORT";
/// The `ZoomLevel` a page role sets after its last report.
const ZOOM_AFTER: &str = "BEVY_CEF_TEST_ZOOM_AFTER";
/// When set, a page role ends with `std::process::exit(0)` 3 s after its report.
const KILL: &str = "BEVY_CEF_TEST_KILL";
/// Prefixes each result line a role prints.
const RESULT_PREFIX: &str = "RESULT ";
/// The line the `holder` role prints once its page has reported.
const READY: &str = "READY";

/// Returns an error built from the format arguments when `cond` is false.
macro_rules! ensure {
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            return Err(format!($($arg)+));
        }
    };
}

fn main() -> ExitCode {
    if cfg!(target_os = "macos") || std::env::var(PROCESS_TESTS).as_deref() != Ok("1") {
        println!("skipped");
        return ExitCode::SUCCESS;
    }
    #[cfg(not(target_os = "macos"))]
    bevy_cef::prelude::early_exit_if_subprocess();
    match std::env::var(ROLE) {
        Ok(role) => role::run(&role),
        Err(_) => scenarios::run_all(),
    }
}

/// What the test page reports about the state it can reach.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PageReport {
    /// The `Cookie` header the server received, HttpOnly cookies included.
    server_cookies: String,
    /// `document.cookie`.
    js_cookies: String,
    /// The localStorage item `token`.
    local_storage: Option<String>,
    /// The IndexedDB record `token`.
    indexed_db: Option<String>,
    device_pixel_ratio: f64,
    inner_width: f64,
}

/// One `RESULT` line printed by a role.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RoleResult {
    /// A report from a page, in arrival order.
    Report(PageReport),
    /// Whether the webview still had no browser when its state was changed.
    ChangedWhileWaiting { waiting: bool },
    /// `Browsers::is_audio_muted` for the current webview.
    Muted { muted: Option<bool> },
    /// The state 2 s after a waiting webview was despawned.
    AfterDespawn {
        live_browsers: usize,
        load_finished: bool,
    },
    /// The role's `CefStatus`.
    Status {
        running: bool,
        root_in_use: bool,
        detail: String,
    },
}

/// The HTTP server the test pages load from.
mod server {
    use std::io::{BufRead, BufReader, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::Duration;

    /// The test page. In `set` mode it stores `token` in two JS cookies, localStorage and
    /// IndexedDB, and sets `late=1` 500 ms after `load`. In both modes it reports once on `load`
    /// and again on each `report` host event.
    const PAGE: &str = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>profile persistence</title></head>
<body>
<script>
const params = new URLSearchParams(location.search);
const token = params.get("token");

function openRecords() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("persistence", 1);
    request.onupgradeneeded = () => request.result.createObjectStore("records");
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function withRecords(mode, use) {
  const db = await openRecords();
  try {
    return await new Promise((resolve, reject) => {
      const transaction = db.transaction("records", mode);
      const request = use(transaction.objectStore("records"));
      transaction.oncomplete = () => resolve(request.result ?? null);
      transaction.onerror = () => reject(transaction.error);
    });
  } finally {
    db.close();
  }
}

async function report() {
  const whoami = await fetch("/whoami", { cache: "no-store" });
  window.cef.emit({
    server_cookies: await whoami.text(),
    js_cookies: document.cookie,
    local_storage: localStorage.getItem("token"),
    indexed_db: await withRecords("readonly", (records) => records.get("token")),
    device_pixel_ratio: window.devicePixelRatio,
    inner_width: window.innerWidth,
  });
}

window.cef.listen("report", () => report());

window.addEventListener("load", async () => {
  if (params.get("mode") === "set") {
    setTimeout(() => { document.cookie = "late=1; Max-Age=31536000; path=/"; }, 500);
    document.cookie = `jsp=${token}; Max-Age=31536000; path=/`;
    document.cookie = `jss=${token}; path=/`;
    localStorage.setItem("token", token);
    await withRecords("readwrite", (records) => records.put(token, "token"));
  }
  await report();
});
</script>
</body>
</html>
"#;

    /// Starts the server on an ephemeral `127.0.0.1` port and returns the port.
    ///
    /// `GET /page?mode=…&token=…` serves the test page, with the `sid` and `ssid` cookies for
    /// `mode=set`; `GET /whoami` echoes the request's `Cookie` header.
    pub fn start() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind the test HTTP server");
        let port = listener
            .local_addr()
            .expect("read the test HTTP server address")
            .port();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                thread::spawn(move || serve(stream));
            }
        });
        port
    }

    fn serve(mut stream: TcpStream) {
        let Some(request) = Request::read(&stream) else {
            return;
        };
        let response = match request.path.as_str() {
            "/page" => Response::page(&request.query),
            "/whoami" => Response::new("200 OK", "text/plain", request.cookie),
            _ => Response::new("404 Not Found", "text/plain", String::new()),
        };
        let _ = response.write(&mut stream);
    }

    struct Request {
        path: String,
        query: String,
        cookie: String,
    }

    impl Request {
        /// Reads the request line and headers, or `None` when the connection carries no
        /// complete request.
        fn read(stream: &TcpStream) -> Option<Self> {
            stream
                .set_read_timeout(Some(Duration::from_secs(10)))
                .ok()?;
            let mut reader = BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).ok()?;
            let target = request_line.split_whitespace().nth(1)?;
            let (path, query) = target.split_once('?').unwrap_or((target, ""));
            let mut cookie = String::new();
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).ok()? == 0 {
                    return None;
                }
                let header = header.trim_end();
                if header.is_empty() {
                    break;
                }
                if let Some((name, value)) = header.split_once(':')
                    && name.eq_ignore_ascii_case("cookie")
                {
                    cookie = value.trim().to_owned();
                }
            }
            Some(Self {
                path: path.to_owned(),
                query: query.to_owned(),
                cookie,
            })
        }
    }

    struct Response {
        status: &'static str,
        content_type: &'static str,
        set_cookies: Vec<String>,
        body: String,
    }

    impl Response {
        fn new(status: &'static str, content_type: &'static str, body: String) -> Self {
            Self {
                status,
                content_type,
                set_cookies: Vec::new(),
                body,
            }
        }

        fn page(query: &str) -> Self {
            let param = |name: &str| {
                query
                    .split('&')
                    .filter_map(|pair| pair.split_once('='))
                    .find_map(|(key, value)| (key == name).then_some(value))
            };
            let mut response = Self::new("200 OK", "text/html; charset=utf-8", PAGE.to_owned());
            if param("mode") == Some("set") {
                let token = param("token").unwrap_or_default();
                response.set_cookies = vec![
                    format!("sid={token}; Max-Age=31536000; HttpOnly"),
                    format!("ssid={token}; HttpOnly"),
                ];
            }
            response
        }

        fn write(&self, stream: &mut TcpStream) -> std::io::Result<()> {
            let mut head = format!(
                "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\
                 Cache-Control: no-store\r\nConnection: close\r\n",
                self.status,
                self.content_type,
                self.body.len()
            );
            for cookie in &self.set_cookies {
                head.push_str(&format!("Set-Cookie: {cookie}\r\n"));
            }
            head.push_str("\r\n");
            stream.write_all(head.as_bytes())?;
            stream.write_all(self.body.as_bytes())
        }
    }
}

/// Spawning role processes and reading their results.
mod child {
    use super::{PORT, PROFILE, PageReport, RESULT_PREFIX, ROLE, ROOT, RoleResult, TOKEN};
    use std::cell::Cell;
    use std::fs;
    use std::io::{BufRead, BufReader, Read};
    use std::path::PathBuf;
    use std::process::{Child, Command, ExitStatus, Stdio};
    use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
    use std::thread;
    use std::time::{Duration, Instant};

    /// How long a role may run before it is killed and its scenario fails.
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
    /// How often a running role is checked.
    const POLL_INTERVAL: Duration = Duration::from_millis(50);
    /// How long output is still collected after a role exits.
    const OUTPUT_GRACE: Duration = Duration::from_secs(2);
    /// How many output lines a failure message shows.
    const TAIL_LINES: usize = 30;

    /// A role process to start: its name, profile and scenario options.
    pub struct Role {
        name: &'static str,
        profile: Option<&'static str>,
        options: Vec<(&'static str, String)>,
        timeout: Duration,
    }

    impl Role {
        pub fn new(name: &'static str) -> Self {
            Self {
                name,
                profile: None,
                options: Vec::new(),
                timeout: DEFAULT_TIMEOUT,
            }
        }

        pub fn profile(mut self, profile: &'static str) -> Self {
            self.profile = Some(profile);
            self
        }

        pub fn token(self, token: &str) -> Self {
            self.option(TOKEN, token)
        }

        pub fn option(mut self, name: &'static str, value: impl ToString) -> Self {
            self.options.push((name, value.to_string()));
            self
        }

        pub fn timeout(mut self, timeout: Duration) -> Self {
            self.timeout = timeout;
            self
        }
    }

    /// The orchestrator's fixed setup: this executable, the fresh root and the server port.
    pub struct Harness {
        exe: PathBuf,
        base: PathBuf,
        root: String,
        port: u16,
        runs: Cell<usize>,
    }

    impl Harness {
        /// Creates a fresh directory under the system temp directory holding the root cache
        /// directory and the roles' logs.
        pub fn new(port: u16) -> Self {
            let base = std::env::temp_dir().join(format!(
                "bevy_cef_profile_persistence_{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(base.join("logs")).expect("create the log directory");
            Self {
                exe: std::env::current_exe().expect("locate the test executable"),
                root: base.join("root").to_string_lossy().into_owned(),
                base,
                port,
                runs: Cell::new(0),
            }
        }

        /// Returns a value unique to `scenario` in this run.
        pub fn token(&self, scenario: &str) -> String {
            format!("{scenario}-{}", std::process::id())
        }

        /// Runs `role` to completion.
        ///
        /// # Errors
        ///
        /// Fails when the role times out, exits with a non-zero code, or prints a malformed
        /// result.
        pub fn run(&self, role: Role) -> Result<Finished, String> {
            self.start(role).finish()
        }

        /// Starts `role` without waiting for it.
        pub fn start(&self, role: Role) -> Running {
            let run = self.runs.get() + 1;
            self.runs.set(run);
            let label = match role.profile {
                Some(profile) => format!("{}({profile})", role.name),
                None => role.name.to_owned(),
            };
            let mut command = Command::new(&self.exe);
            command
                .env(ROLE, role.name)
                .env(ROOT, &self.root)
                .env(PORT, self.port.to_string())
                .envs(role.options.iter().map(|(name, value)| (name, value)))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            if let Some(profile) = role.profile {
                command.env(PROFILE, profile);
            }
            let mut child = command.spawn().expect("spawn a role process");
            let (sender, lines) = mpsc::channel();
            forward(child.stdout.take(), Line::Out, sender.clone());
            forward(child.stderr.take(), Line::Err, sender);
            Running {
                log: self.logs().join(format!("{run:02}-{}.log", role.name)),
                label,
                child,
                lines,
                output: Vec::new(),
                timeout: role.timeout,
                deadline: Instant::now() + role.timeout,
            }
        }

        pub fn logs(&self) -> PathBuf {
            self.base.join("logs")
        }

        /// Removes the root cache directory and the logs, ignoring failures.
        pub fn remove(&self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }

    /// A role process that has been started.
    ///
    /// Dropping it kills the process if it is still running.
    pub struct Running {
        label: String,
        child: Child,
        lines: Receiver<Line>,
        output: Vec<Line>,
        timeout: Duration,
        deadline: Instant,
        log: PathBuf,
    }

    impl Running {
        /// Waits until the role prints the stdout line `wanted`.
        ///
        /// # Errors
        ///
        /// Fails when the role exits first or times out; a timed-out role is killed.
        pub fn wait_for_line(&mut self, wanted: &str) -> Result<(), String> {
            while !self.stdout().any(|line| line == wanted) {
                if let Some(status) = self.poll_exit()? {
                    return Err(self.failure(&format!("exited with {status} before {wanted}")));
                }
            }
            Ok(())
        }

        /// Waits for the role to exit and parses its `RESULT` lines.
        ///
        /// # Errors
        ///
        /// Fails when the role times out (it is then killed), exits with a non-zero code, or
        /// prints a malformed result.
        pub fn finish(mut self) -> Result<Finished, String> {
            let status = loop {
                if let Some(status) = self.poll_exit()? {
                    break status;
                }
            };
            self.drain_after_exit();
            if !status.success() {
                return Err(self.failure(&format!("exited with {status}")));
            }
            let results = self
                .stdout()
                .filter_map(|line| line.strip_prefix(RESULT_PREFIX))
                .map(serde_json::from_str::<RoleResult>)
                .collect::<Result<Vec<_>, _>>();
            match results {
                Ok(results) => {
                    self.save_log();
                    Ok(Finished {
                        label: std::mem::take(&mut self.label),
                        results,
                        log: self.log.clone(),
                    })
                }
                Err(error) => Err(self.failure(&format!("printed a malformed result: {error}"))),
            }
        }

        fn stdout(&self) -> impl Iterator<Item = &str> {
            self.output.iter().filter_map(|line| match line {
                Line::Out(text) => Some(text.as_str()),
                Line::Err(_) => None,
            })
        }

        /// Collects output for one poll interval, then returns the exit status if the role has
        /// exited.
        ///
        /// # Errors
        ///
        /// Kills the role and fails once its deadline has passed.
        fn poll_exit(&mut self) -> Result<Option<ExitStatus>, String> {
            match self.lines.recv_timeout(POLL_INTERVAL) {
                Ok(line) => self.output.push(line),
                Err(RecvTimeoutError::Disconnected) => thread::sleep(POLL_INTERVAL),
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.output.extend(self.lines.try_iter());
            match self.child.try_wait() {
                Ok(Some(status)) => Ok(Some(status)),
                Ok(None) if Instant::now() < self.deadline => Ok(None),
                Ok(None) => {
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    self.drain_after_exit();
                    Err(self.failure(&format!(
                        "timed out after {:?} and was killed",
                        self.timeout
                    )))
                }
                Err(error) => Err(self.failure(&format!("could not be waited on: {error}"))),
            }
        }

        /// Collects output until every writer has closed its pipe or the grace period ends.
        fn drain_after_exit(&mut self) {
            let deadline = Instant::now() + OUTPUT_GRACE;
            while let Ok(line) = self
                .lines
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                self.output.push(line);
            }
        }

        /// Saves the log and returns `problem` with the log path and the output's tail.
        fn failure(&self, problem: &str) -> String {
            self.save_log();
            let skip = self.output.len().saturating_sub(TAIL_LINES);
            let tail: Vec<String> = self.output[skip..]
                .iter()
                .map(|line| format!("    {line}"))
                .collect();
            format!(
                "{}: {problem}\n  log: {}\n  last output:\n{}",
                self.label,
                self.log.display(),
                tail.join("\n")
            )
        }

        fn save_log(&self) {
            let text: Vec<String> = self.output.iter().map(Line::to_string).collect();
            let _ = fs::write(&self.log, text.join("\n"));
        }
    }

    impl Drop for Running {
        fn drop(&mut self) {
            if let Ok(None) = self.child.try_wait() {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }

    /// The results of a role that exited with code 0.
    pub struct Finished {
        label: String,
        results: Vec<RoleResult>,
        log: PathBuf,
    }

    impl Finished {
        /// Returns the `index`th page report, counting from 0.
        pub fn report(&self, index: usize) -> Result<&PageReport, String> {
            self.results
                .iter()
                .filter_map(|result| match result {
                    RoleResult::Report(report) => Some(report),
                    _ => None,
                })
                .nth(index)
                .ok_or_else(|| self.missing(&format!("page report #{}", index + 1)))
        }

        /// Returns the first result `pick` accepts; `what` names it in the error.
        pub fn result<T>(
            &self,
            what: &str,
            pick: impl Fn(&RoleResult) -> Option<T>,
        ) -> Result<T, String> {
            self.results
                .iter()
                .find_map(pick)
                .ok_or_else(|| self.missing(what))
        }

        fn missing(&self, what: &str) -> String {
            format!(
                "{}: printed no {what}; results: {:?}\n  log: {}",
                self.label,
                self.results,
                self.log.display()
            )
        }
    }

    /// One line of a role's output.
    enum Line {
        Out(String),
        Err(String),
    }

    impl std::fmt::Display for Line {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Out(text) => write!(f, "out| {text}"),
                Self::Err(text) => write!(f, "err| {text}"),
            }
        }
    }

    /// Sends each line of `source` to `lines`, decoding invalid UTF-8 lossily so a stray byte
    /// never stops the reader and fills the pipe.
    fn forward(
        source: Option<impl Read + Send + 'static>,
        wrap: fn(String) -> Line,
        lines: Sender<Line>,
    ) {
        let Some(source) = source else {
            return;
        };
        thread::spawn(move || {
            let mut reader = BufReader::new(source);
            let mut buffer = Vec::new();
            while reader
                .read_until(b'\n', &mut buffer)
                .is_ok_and(|read| read > 0)
            {
                let text = String::from_utf8_lossy(&buffer).trim_end().to_owned();
                if lines.send(wrap(text)).is_err() {
                    return;
                }
                buffer.clear();
            }
        });
    }
}

/// The role apps: Bevy apps with a hidden window that drive test-page webviews through a list of
/// steps.
mod role {
    use super::{
        FLUSH_SECS, KILL, PORT, PROFILE, PageReport, READY, REREPORT, RESULT_PREFIX, ROOT,
        RoleResult, TOKEN, ZOOM, ZOOM_AFTER,
    };
    use bevy::prelude::*;
    use bevy::window::{ExitCondition, WindowResolution};
    use bevy_cef::prelude::*;
    use bevy_cef_core::prelude::Browsers;
    use std::collections::{HashSet, VecDeque};
    use std::process::{ExitCode, Termination};
    use std::time::{Duration, Instant};

    /// Runs the role `name` and returns its exit code.
    pub fn run(name: &str) -> ExitCode {
        let env = RoleEnv::from_env();
        let script = Script {
            steps: env.steps(name).into(),
            step_started: Instant::now(),
            page: format!("http://127.0.0.1:{}/page?token={}", env.port, env.token),
            profile: env.profile.clone(),
            webview: None,
            despawned: None,
            exit_requested: false,
        };
        // NOTE: The winit runner pumps the native message loop that CEF's process singleton
        // answers a second process through; without it the second process waits, then kills
        // this one as hung. The window stays hidden, and its scale factor is pinned so the
        // monitor's DPI does not change `devicePixelRatio`.
        App::new()
            .add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        visible: false,
                        resolution: WindowResolution::default().with_scale_factor_override(1.0),
                        ..default()
                    }),
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                }),
                CefPlugin {
                    root_cache_path: Some(env.root.clone()),
                    cookie_flush_interval: env.flush_interval,
                    ..default()
                },
                JsEmitEventPlugin::<PageReport>::default(),
            ))
            .insert_resource(script)
            .init_resource::<Observed>()
            .add_observer(print_report)
            .add_observer(note_load_finished)
            .add_systems(First, run_script)
            .add_systems(Last, exit_when_requested)
            .run()
            .report()
    }

    /// The role's settings, read from the environment.
    struct RoleEnv {
        root: String,
        port: u16,
        profile: Option<WebviewProfile>,
        token: String,
        flush_interval: Duration,
        zoom: f64,
        rereport: bool,
        zoom_after: Option<f64>,
        kill: bool,
    }

    impl RoleEnv {
        fn from_env() -> Self {
            let var = |name: &str| std::env::var(name).ok();
            let number = |name: &str| {
                var(name).map(|value| {
                    value
                        .parse::<f64>()
                        .unwrap_or_else(|_| panic!("{name} must be a number, got {value:?}"))
                })
            };
            Self {
                root: var(ROOT).expect("the orchestrator sets the root"),
                port: number(PORT).expect("the orchestrator sets the port") as u16,
                profile: var(PROFILE)
                    .map(|name| WebviewProfile::new(name).expect("a valid profile name")),
                token: var(TOKEN).unwrap_or_default(),
                flush_interval: Duration::from_secs_f64(number(FLUSH_SECS).unwrap_or(1.0)),
                zoom: number(ZOOM).unwrap_or(0.0),
                rereport: var(REREPORT).is_some(),
                zoom_after: number(ZOOM_AFTER),
                kill: var(KILL).is_some(),
            }
        }

        /// Returns the steps of the role `name`.
        fn steps(&self, name: &str) -> Vec<Step> {
            match name {
                "set" => self.page_steps(Mode::Set),
                "get" => self.page_steps(Mode::Get),
                "shared" => vec![
                    Step::spawn(Mode::Set),
                    Step::AwaitReports(1),
                    Step::spawn(Mode::Get),
                    Step::AwaitReports(2),
                    Step::Exit,
                ],
                "state_while_pending" => {
                    let mut steps = vec![
                        Step::Spawn {
                            mode: Mode::Get,
                            zoom: 0.0,
                            size: Vec2::new(800., 600.),
                        },
                        Step::NextFrame,
                        Step::ChangeWhileWaiting,
                        Step::AwaitReports(1),
                    ];
                    steps.extend(Step::rereport());
                    steps.extend([Step::RecordMuted, Step::Exit]);
                    steps
                }
                "exit_while_pending" => vec![Step::spawn(Mode::Get), Step::ExitNow],
                "despawn_then_respawn" => vec![
                    Step::spawn(Mode::Get),
                    Step::NextFrame,
                    Step::Despawn,
                    Step::Sleep(Duration::from_secs(2)),
                    Step::RecordAfterDespawn,
                    Step::spawn(Mode::Get),
                    Step::AwaitReports(1),
                    Step::Exit,
                ],
                "holder" => vec![
                    Step::spawn(Mode::Get),
                    Step::AwaitReports(1),
                    Step::PrintReady,
                    Step::Sleep(Duration::from_secs(8)),
                    Step::Exit,
                ],
                "second" => vec![Step::RecordStatus, Step::Exit],
                other => panic!("unknown role {other:?}"),
            }
        }

        /// One page webview that reports, optionally reports again and changes its zoom, and
        /// then exits 1 s later through `AppExit` or is killed 3 s later.
        fn page_steps(&self, mode: Mode) -> Vec<Step> {
            let mut steps = vec![
                Step::Spawn {
                    mode,
                    zoom: self.zoom,
                    size: WebviewSize::default().0,
                },
                Step::AwaitReports(1),
            ];
            if self.rereport {
                steps.extend(Step::rereport());
            }
            if let Some(level) = self.zoom_after {
                steps.push(Step::SetZoom(level));
            }
            if self.kill {
                steps.extend([Step::Sleep(Duration::from_secs(3)), Step::Kill]);
            } else {
                // The settle time outlasts the `late` cookie the page sets 500 ms after `load`.
                steps.extend([Step::Sleep(Duration::from_secs(1)), Step::Exit]);
            }
            steps
        }
    }

    /// Which page a webview loads.
    #[derive(Clone, Copy)]
    enum Mode {
        /// Stores state, then reports.
        Set,
        /// Only reports.
        Get,
    }

    /// One step of a role. Every step after `Spawn` acts on the most recently spawned webview.
    #[derive(Clone)]
    enum Step {
        /// Spawns a test-page webview with the role's profile.
        Spawn {
            mode: Mode,
            zoom: f64,
            size: Vec2,
        },
        /// Continues in the next frame.
        NextFrame,
        Sleep(Duration),
        /// Waits until this many reports have arrived in total.
        AwaitReports(usize),
        AwaitLoadFinished,
        /// Asks the page to report again.
        RequestReport,
        SetZoom(f64),
        /// Sets the size to 640×480, mutes the audio and sets the zoom to 1.0, and records
        /// whether the webview still had no browser.
        ChangeWhileWaiting,
        Despawn,
        RecordMuted,
        /// Records the live-browser count and whether the despawned webview finished a load.
        RecordAfterDespawn,
        RecordStatus,
        PrintReady,
        /// Exits through a `Last` system writing `AppExit`.
        Exit,
        /// Writes `AppExit` in this frame.
        ExitNow,
        /// Exits the process without `AppExit`.
        Kill,
    }

    impl Step {
        fn spawn(mode: Mode) -> Self {
            Self::Spawn {
                mode,
                zoom: 0.0,
                size: WebviewSize::default().0,
            }
        }

        /// Asks for a second report 500 ms after `LoadFinished` and waits for it.
        fn rereport() -> [Self; 4] {
            [
                Self::AwaitLoadFinished,
                Self::Sleep(Duration::from_millis(500)),
                Self::RequestReport,
                Self::AwaitReports(2),
            ]
        }
    }

    /// What a step does to the script.
    enum Flow {
        /// The step is done; the next one runs in this frame.
        Next,
        /// The step is done; the next one runs in the next frame.
        NextFrame,
        /// The step is not done yet.
        Wait,
    }

    #[derive(Resource)]
    struct Script {
        steps: VecDeque<Step>,
        step_started: Instant,
        /// The test page URL without its `mode` parameter.
        page: String,
        profile: Option<WebviewProfile>,
        webview: Option<Entity>,
        despawned: Option<Entity>,
        exit_requested: bool,
    }

    impl Script {
        fn advance(&mut self) {
            self.steps.pop_front();
            self.step_started = Instant::now();
        }

        fn webview(&self) -> Entity {
            self.webview.expect("a step acts on a spawned webview")
        }
    }

    /// The events the role has seen.
    #[derive(Resource, Default)]
    struct Observed {
        reports: usize,
        load_finished: HashSet<Entity>,
    }

    /// Runs the script's steps until one has to wait.
    ///
    /// Runs in `First`, so a webview spawned here resolves its source and requests its browser
    /// within the same frame.
    fn run_script(
        mut commands: Commands,
        mut script: ResMut<Script>,
        mut exit: MessageWriter<AppExit>,
        mut webviews: Query<(&mut WebviewSize, &mut AudioMuted, &mut ZoomLevel)>,
        browsers: Option<NonSend<Browsers>>,
        observed: Res<Observed>,
        status: Res<CefStatus>,
    ) {
        while let Some(step) = script.steps.front().cloned() {
            let flow = match step {
                Step::Spawn { mode, zoom, size } => {
                    let mode = match mode {
                        Mode::Set => "set",
                        Mode::Get => "get",
                    };
                    let mut webview = commands.spawn((
                        WebviewSource::new(format!("{}&mode={mode}", script.page)),
                        ZoomLevel(zoom),
                        WebviewSize(size),
                    ));
                    if let Some(profile) = &script.profile {
                        webview.insert(profile.clone());
                    }
                    script.webview = Some(webview.id());
                    Flow::Next
                }
                Step::NextFrame => Flow::NextFrame,
                Step::Sleep(duration) => wait_until(script.step_started.elapsed() >= duration),
                Step::AwaitReports(count) => wait_until(observed.reports >= count),
                Step::AwaitLoadFinished => {
                    wait_until(observed.load_finished.contains(&script.webview()))
                }
                Step::RequestReport => {
                    commands.trigger(HostEmitEvent::new(script.webview(), "report", &()));
                    Flow::Next
                }
                Step::SetZoom(level) => {
                    let (_, _, mut zoom) = webviews
                        .get_mut(script.webview())
                        .expect("the webview exists");
                    zoom.0 = level;
                    Flow::Next
                }
                Step::ChangeWhileWaiting => {
                    let webview = script.webview();
                    let waiting = browsers
                        .as_deref()
                        .expect("CEF is running")
                        .is_audio_muted(&webview)
                        .is_none();
                    let (mut size, mut muted, mut zoom) =
                        webviews.get_mut(webview).expect("the webview exists");
                    size.0 = Vec2::new(640., 480.);
                    muted.0 = true;
                    zoom.0 = 1.0;
                    print_result(&RoleResult::ChangedWhileWaiting { waiting });
                    Flow::Next
                }
                Step::Despawn => {
                    let webview = script.webview();
                    commands.entity(webview).despawn();
                    script.despawned = Some(webview);
                    script.webview = None;
                    Flow::Next
                }
                Step::RecordMuted => {
                    let muted = browsers
                        .as_deref()
                        .expect("CEF is running")
                        .is_audio_muted(&script.webview());
                    print_result(&RoleResult::Muted { muted });
                    Flow::Next
                }
                Step::RecordAfterDespawn => {
                    let despawned = script.despawned.expect("a webview was despawned");
                    print_result(&RoleResult::AfterDespawn {
                        live_browsers: browsers
                            .as_deref()
                            .expect("CEF is running")
                            .live_browser_count(),
                        load_finished: observed.load_finished.contains(&despawned),
                    });
                    Flow::Next
                }
                Step::RecordStatus => {
                    print_result(&match &*status {
                        CefStatus::Running => RoleResult::Status {
                            running: true,
                            root_in_use: false,
                            detail: String::new(),
                        },
                        CefStatus::Failed(error) => RoleResult::Status {
                            running: false,
                            root_in_use: error.is_root_in_use(),
                            detail: error.to_string(),
                        },
                    });
                    Flow::Next
                }
                Step::PrintReady => {
                    println!("{READY}");
                    Flow::Next
                }
                Step::Exit => {
                    script.exit_requested = true;
                    Flow::Next
                }
                Step::ExitNow => {
                    exit.write(AppExit::Success);
                    Flow::Next
                }
                Step::Kill => std::process::exit(0),
            };
            match flow {
                Flow::Next => script.advance(),
                Flow::NextFrame => {
                    script.advance();
                    return;
                }
                Flow::Wait => return,
            }
        }
    }

    /// Writes `AppExit` from `Last`, the schedule a window close writes it from.
    fn exit_when_requested(script: Res<Script>, mut exit: MessageWriter<AppExit>) {
        if script.exit_requested {
            exit.write(AppExit::Success);
        }
    }

    fn print_report(report: On<Receive<PageReport>>, mut observed: ResMut<Observed>) {
        observed.reports += 1;
        print_result(&RoleResult::Report(report.payload.clone()));
    }

    fn note_load_finished(finished: On<LoadFinished>, mut observed: ResMut<Observed>) {
        observed.load_finished.insert(finished.webview);
    }

    fn wait_until(done: bool) -> Flow {
        if done { Flow::Next } else { Flow::Wait }
    }

    fn print_result(result: &RoleResult) {
        let json = serde_json::to_string(result).expect("serialize a role result");
        println!("{RESULT_PREFIX}{json}");
    }
}

/// The scenarios and the orchestrator that runs them.
mod scenarios {
    use super::child::{Harness, Role};
    use super::{
        FLUSH_SECS, KILL, PageReport, READY, REREPORT, RoleResult, ZOOM, ZOOM_AFTER, server,
    };
    use std::collections::BTreeMap;
    use std::process::ExitCode;
    use std::time::{Duration, Instant};

    type Scenario = fn(&Harness) -> Result<(), String>;

    /// Every scenario, in the order they run. Later scenarios reuse profile `a`, so each writes a
    /// token of its own.
    const SCENARIOS: [(&str, Scenario); 10] = [
        ("persist", persist),
        ("isolation", isolation),
        ("shared_in_process", shared_in_process),
        ("no_profile_keeps_nothing", no_profile_keeps_nothing),
        ("kill_after_flush", kill_after_flush),
        ("zoom", zoom),
        ("state_while_pending", state_while_pending),
        ("exit_while_pending", exit_while_pending),
        ("despawn_then_respawn", despawn_then_respawn),
        ("second_process", second_process),
    ];

    /// Runs every scenario whose name contains the first non-flag argument, if any, and fails
    /// when one fails. The root and logs are removed only when all pass.
    pub fn run_all() -> ExitCode {
        let filter = std::env::args().skip(1).find(|arg| !arg.starts_with('-'));
        let harness = Harness::new(server::start());
        let started = Instant::now();
        let mut passed = 0;
        let mut failed = Vec::new();
        for (name, scenario) in SCENARIOS {
            if filter
                .as_deref()
                .is_some_and(|filter| !name.contains(filter))
            {
                continue;
            }
            let scenario_started = Instant::now();
            match scenario(&harness) {
                Ok(()) => {
                    passed += 1;
                    println!(
                        "test {name} ... ok ({:.1}s)",
                        scenario_started.elapsed().as_secs_f64()
                    );
                }
                Err(error) => {
                    println!("test {name} ... FAILED\n{error}");
                    failed.push(name);
                }
            }
        }
        println!(
            "\n{passed} passed; {} failed; finished in {:.1}s",
            failed.len(),
            started.elapsed().as_secs_f64()
        );
        if failed.is_empty() {
            harness.remove();
            ExitCode::SUCCESS
        } else {
            println!(
                "failed: {failed:?}; role logs: {}",
                harness.logs().display()
            );
            ExitCode::FAILURE
        }
    }

    /// Cookies (HttpOnly, session and JS), localStorage and IndexedDB survive a restart in a
    /// profile. The `late` cookie is set after the load-end flush, and the hour-long interval
    /// never fires, so only the `shutdown()` of the window-close exit path can have written it.
    fn persist(harness: &Harness) -> Result<(), String> {
        let token = harness.token("persist");
        harness.run(
            Role::new("set")
                .profile("a")
                .token(&token)
                .option(FLUSH_SECS, 3600),
        )?;
        let get = harness.run(Role::new("get").profile("a"))?;
        let report = get.report(0)?;
        let cookies = cookie_map(&report.server_cookies);
        for name in ["sid", "ssid", "jsp", "jss"] {
            ensure!(
                cookies.get(name) == Some(&token.as_str()),
                "cookie {name} did not survive the restart: {report:?}"
            );
        }
        ensure!(
            cookies.get("late") == Some(&"1"),
            "the late cookie, which only shutdown() can have written, was lost: {report:?}"
        );
        ensure_storage(report, &token)
    }

    /// A different profile name sees none of profile `a`'s state.
    fn isolation(harness: &Harness) -> Result<(), String> {
        let get = harness.run(Role::new("get").profile("b"))?;
        ensure_nothing_kept(get.report(0)?)
    }

    /// Two webviews of one profile in one process share localStorage.
    fn shared_in_process(harness: &Harness) -> Result<(), String> {
        let token = harness.token("shared_in_process");
        let shared = harness.run(Role::new("shared").profile("a").token(&token))?;
        let second = shared.report(1)?;
        ensure!(
            second.local_storage.as_deref() == Some(token.as_str()),
            "the second webview did not read the localStorage the first wrote: {second:?}"
        );
        Ok(())
    }

    /// Webviews without a profile keep nothing across a restart.
    fn no_profile_keeps_nothing(harness: &Harness) -> Result<(), String> {
        let token = harness.token("no_profile_keeps_nothing");
        let set = harness.run(Role::new("set").token(&token))?;
        let stored = set.report(0)?;
        ensure!(
            cookie_map(&stored.server_cookies).get("sid") == Some(&token.as_str()),
            "the set page did not store its state: {stored:?}"
        );
        ensure_storage(stored, &token)?;
        let get = harness.run(Role::new("get"))?;
        ensure_nothing_kept(get.report(0)?)
    }

    /// A cookie stored before a completed interval or load-end flush survives a kill.
    fn kill_after_flush(harness: &Harness) -> Result<(), String> {
        let token = harness.token("kill_after_flush");
        harness.run(Role::new("set").profile("k").token(&token).option(KILL, 1))?;
        let get = harness.run(Role::new("get").profile("k"))?;
        let report = get.report(0)?;
        ensure!(
            cookie_map(&report.server_cookies).get("sid") == Some(&token.as_str()),
            "sid was lost to the kill after a flush: {report:?}"
        );
        Ok(())
    }

    /// A spawn-time zoom is applied after the first commit, and a default `ZoomLevel` leaves
    /// the profile's remembered site zoom in place after a restart.
    fn zoom(harness: &Harness) -> Result<(), String> {
        let token = harness.token("zoom");
        let set = harness.run(
            Role::new("set")
                .profile("z")
                .token(&token)
                .option(ZOOM, 1.0)
                .option(REREPORT, 1)
                .option(ZOOM_AFTER, 2.0),
        )?;
        ensure_device_pixel_ratio(set.report(1)?, 1.2)?;
        let get = harness.run(Role::new("get").profile("z").option(REREPORT, 1))?;
        ensure_device_pixel_ratio(get.report(1)?, 1.44)
    }

    /// Size, mute and zoom changed while a profile webview waits for its context are applied
    /// once its browser exists.
    fn state_while_pending(harness: &Harness) -> Result<(), String> {
        let run = harness.run(Role::new("state_while_pending").profile("a"))?;
        let waiting = run.result("changed_while_waiting result", |result| match result {
            RoleResult::ChangedWhileWaiting { waiting } => Some(*waiting),
            _ => None,
        })?;
        ensure!(
            waiting,
            "the browser already existed when the state changed"
        );
        let first = run.report(0)?;
        let width_in_dips = first.inner_width * first.device_pixel_ratio;
        let size_applied = (width_in_dips - 640.0).abs() < 1.0;
        ensure!(
            size_applied,
            "the size set while waiting was not applied (DIPs, inner_width × device_pixel_ratio \
             = {width_in_dips}): {first:?}"
        );
        let muted = run.result("muted result", |result| match result {
            RoleResult::Muted { muted } => Some(*muted),
            _ => None,
        })?;
        ensure!(
            muted == Some(true),
            "the mute set while waiting was not applied: {muted:?}"
        );
        ensure_device_pixel_ratio(run.report(1)?, 1.2)
    }

    /// Exiting while a profile webview waits for its context still exits cleanly.
    fn exit_while_pending(harness: &Harness) -> Result<(), String> {
        harness.run(
            Role::new("exit_while_pending")
                .profile("a")
                .timeout(Duration::from_secs(10)),
        )?;
        Ok(())
    }

    /// Despawning a waiting webview leaves no browser behind, and the profile still serves a
    /// webview spawned afterwards.
    fn despawn_then_respawn(harness: &Harness) -> Result<(), String> {
        let run = harness.run(Role::new("despawn_then_respawn").profile("a"))?;
        let (live_browsers, load_finished) =
            run.result("after_despawn result", |result| match result {
                RoleResult::AfterDespawn {
                    live_browsers,
                    load_finished,
                } => Some((*live_browsers, *load_finished)),
                _ => None,
            })?;
        ensure!(
            live_browsers == 0,
            "{live_browsers} browser(s) live after despawning the waiting webview"
        );
        ensure!(!load_finished, "the despawned webview finished a page load");
        run.report(0)?;
        Ok(())
    }

    /// A second process on the same root reports that the root is in use, and both processes
    /// exit cleanly.
    fn second_process(harness: &Harness) -> Result<(), String> {
        let mut holder = harness.start(Role::new("holder").profile("a"));
        holder.wait_for_line(READY)?;
        let second = harness.run(Role::new("second"));
        let holder = holder.finish();
        let second = second?;
        holder?;
        let (running, root_in_use, detail) =
            second.result("status result", |result| match result {
                RoleResult::Status {
                    running,
                    root_in_use,
                    detail,
                } => Some((*running, *root_in_use, detail.clone())),
                _ => None,
            })?;
        ensure!(
            !running && root_in_use,
            "the second process did not report the root in use: running {running}, {detail}"
        );
        Ok(())
    }

    /// Parses a `Cookie` header or `document.cookie` into names and values.
    fn cookie_map(cookies: &str) -> BTreeMap<&str, &str> {
        cookies
            .split(';')
            .filter_map(|pair| pair.trim().split_once('='))
            .collect()
    }

    fn ensure_storage(report: &PageReport, token: &str) -> Result<(), String> {
        ensure!(
            report.local_storage.as_deref() == Some(token),
            "localStorage did not hold the token: {report:?}"
        );
        ensure!(
            report.indexed_db.as_deref() == Some(token),
            "IndexedDB did not hold the token: {report:?}"
        );
        Ok(())
    }

    fn ensure_nothing_kept(report: &PageReport) -> Result<(), String> {
        ensure!(
            report.server_cookies.is_empty()
                && report.js_cookies.is_empty()
                && report.local_storage.is_none()
                && report.indexed_db.is_none(),
            "expected no stored state: {report:?}"
        );
        Ok(())
    }

    fn ensure_device_pixel_ratio(report: &PageReport, expected: f64) -> Result<(), String> {
        let within_tolerance = (report.device_pixel_ratio - expected).abs() <= 0.01;
        ensure!(
            within_tolerance,
            "expected device_pixel_ratio ≈ {expected}: {report:?}"
        );
        Ok(())
    }
}
