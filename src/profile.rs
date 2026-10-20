use crate::common::{
    AudioMuted, CefRootCachePath, CefShutDown, HostWindow, ResolvedWebviewUri, WebviewDpr,
    WebviewSize, WebviewSource, ZoomLevel,
};
use crate::focus::FocusedWebview;
use crate::navigation::AddressChanged;
use crate::prelude::PreloadScripts;
use crate::webview::{BrowserCreation, CreateArgs, WebviewSet};
use bevy::ecs::change_detection::Tick;
use bevy::ecs::system::SystemChangeTick;
use bevy::prelude::*;
use bevy::time::common_conditions::on_real_timer;
use bevy_cef_core::prelude::{Browsers, ProfileContextError, Requester};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{DirBuilder, File, remove_file};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Creates the browsers of webviews that carry a [`WebviewProfile`] on their profile's shared,
/// disk-backed request context, and flushes the cookie stores of those contexts.
pub(crate) struct ProfilePlugin {
    pub(crate) cookie_flush_interval: Duration,
}

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WebviewProfileError>()
            .add_systems(
                Update,
                (
                    request_profile_browsers
                        .run_if(added_profile_webview)
                        .run_if(not(resource_exists::<CefShutDown>)),
                    // NOTE: The executor evaluates run conditions on whichever thread ticks it,
                    // so a condition must not read the `NonSend` `Browsers`; doing so panics off
                    // the main thread. The system itself checks which contexts are ready.
                    create_ready_profile_browsers
                        .run_if(any_with_component::<PendingProfileBrowser>)
                        .run_if(not(resource_exists::<CefShutDown>)),
                )
                    .chain()
                    .in_set(WebviewSet::CreateBrowser),
            )
            .add_systems(
                Update,
                (
                    flush_profile_cookies
                        .run_if(on_real_timer(self.cookie_flush_interval))
                        .run_if(not(resource_exists::<CefShutDown>)),
                    reapply_mute_after_first_commit
                        .run_if(any_with_component::<MuteAfterFirstCommit>)
                        .run_if(not(resource_exists::<CefShutDown>)),
                    warn_late_profiles.run_if(not(resource_exists::<CefShutDown>)),
                ),
            )
            .add_observer(note_first_commit);
    }
}

/// Reports that a webview with a [`WebviewProfile`] gets no browser.
///
/// Triggered on the webview entity. The webview is left without a browser; bevy_cef never falls
/// back to in-memory storage.
#[derive(Debug, EntityEvent, Clone, Reflect, Serialize, Deserialize)]
pub struct WebviewProfileError {
    #[event_target]
    pub webview: Entity,
    pub reason: WebviewProfileErrorReason,
}

/// Why a webview with a [`WebviewProfile`] gets no browser.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum WebviewProfileErrorReason {
    /// The profile name is not a valid [`ProfileName`]; holds the validation message.
    InvalidName(String),
    /// `CefPlugin::root_cache_path` is unset, blank or not an absolute path.
    NoRoot,
    /// The profile directory could not be created or written; holds the I/O error message.
    Directory(String),
    /// CEF rejected the profile directory and would have kept the profile in memory.
    DidNotLoad,
    /// CEF did not create the profile's request context.
    ContextCreation,
    /// CEF did not create the browser.
    BrowserCreation,
}

/// Marks a profile webview whose browser waits for its profile's context to finish
/// initializing.
///
/// Removing the component (or despawning the entity) cancels the wait.
#[derive(Component, Debug)]
pub(crate) struct PendingProfileBrowser {
    cache_path: String,
    /// The latest `RequestNavigate` URL, with the change tick of the observer that stored it.
    navigation: Option<(String, Tick)>,
}

impl PendingProfileBrowser {
    /// Records `url` as the latest navigation request, made at `requested_at`.
    pub(crate) fn store_navigation(&mut self, url: String, requested_at: Tick) {
        self.navigation = Some((url, requested_at));
    }
}

/// Marks a profile webview whose [`AudioMuted`] is applied again once its first main-frame
/// navigation has committed, since Chromium resets a browser's mute at that commit.
///
/// Removed after the mute is re-applied, so later commits are not affected.
#[derive(Component, Debug, Default)]
pub(crate) struct MuteAfterFirstCommit {
    committed: bool,
}

/// Returns the URL a waiting profile webview's browser is created with, or `None` while the
/// latest `WebviewSource` change has not been resolved yet.
///
/// `source_changed` is the `WebviewSource` change tick and `resolved_changed` the change tick of
/// `resolved`, which is resolved from the source one schedule later. Once the source is
/// resolved, the stored navigation URL wins unless the source changed after the navigation's
/// tick, in which case `resolved` wins. `this_run` is the calling system's change tick.
pub(crate) fn initial_url<'a>(
    resolved: &'a str,
    resolved_changed: Tick,
    navigation: Option<&'a (String, Tick)>,
    source_changed: Tick,
    this_run: Tick,
) -> Option<&'a str> {
    if source_changed.is_newer_than(resolved_changed, this_run) {
        return None;
    }
    Some(match navigation {
        Some((url, navigated)) if !source_changed.is_newer_than(*navigated, this_run) => url,
        _ => resolved,
    })
}

/// A validated profile name: 1 to [`ProfileName::MAX_LEN`] characters drawn from
/// `a`-`z`, `0`-`9`, `-` and `_`.
///
/// Names are rejected when invalid, never rewritten. Deserializing validates the
/// name, so reflection-based writes (BRP) go through the same check.
#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[reflect(opaque, Clone, Debug, Hash, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProfileName(String);

impl ProfileName {
    /// The maximum length of a profile name, in characters.
    pub const MAX_LEN: usize = 64;

    /// Returns the name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Re-checks the inner string, which reflection can overwrite with an invalid value.
    pub(crate) fn revalidate(&self) -> Result<(), ProfileNameError> {
        Self::validate(&self.0)
    }

    /// Wraps `name` without validating it, as a reflection write would.
    #[cfg(test)]
    pub(crate) fn unchecked(name: &str) -> Self {
        Self(name.to_owned())
    }

    fn validate(name: &str) -> Result<(), ProfileNameError> {
        if name.is_empty() {
            return Err(ProfileNameError::Empty);
        }
        if let Some(ch) = name
            .chars()
            .find(|c| !matches!(c, 'a'..='z' | '0'..='9' | '-' | '_'))
        {
            return Err(ProfileNameError::InvalidChar { ch });
        }
        if name.len() > Self::MAX_LEN {
            return Err(ProfileNameError::TooLong { len: name.len() });
        }
        Ok(())
    }
}

impl TryFrom<String> for ProfileName {
    type Error = ProfileNameError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Self::validate(&name)?;
        Ok(Self(name))
    }
}

impl From<ProfileName> for String {
    fn from(name: ProfileName) -> Self {
        name.0
    }
}

/// The reason a string is not a valid [`ProfileName`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileNameError {
    /// The name is empty.
    Empty,
    /// The name is longer than [`ProfileName::MAX_LEN`].
    TooLong {
        /// The length of the rejected name.
        len: usize,
    },
    /// The name contains a character outside `a`-`z`, `0`-`9`, `-` and `_`.
    InvalidChar {
        /// The first offending character.
        ch: char,
    },
}

impl fmt::Display for ProfileNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "profile name is empty"),
            Self::TooLong { len } => write!(
                f,
                "profile name is {len} characters long; the limit is {}",
                ProfileName::MAX_LEN
            ),
            Self::InvalidChar { ch } => write!(
                f,
                "profile name contains {ch:?}; only a-z, 0-9, '-' and '_' are allowed"
            ),
        }
    }
}

impl std::error::Error for ProfileNameError {}

/// Selects a named, disk-backed CEF profile for a webview.
///
/// The component is read once, when the webview's browser is requested. Changing
/// it afterwards, inserting it after that point, and removing it have no effect.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(Component, Debug, Serialize, Deserialize)]
pub struct WebviewProfile(ProfileName);

impl WebviewProfile {
    /// Creates a profile selector from `name`.
    ///
    /// # Errors
    ///
    /// Returns the [`ProfileNameError`] when `name` is not a valid [`ProfileName`].
    pub fn new(name: impl Into<String>) -> Result<Self, ProfileNameError> {
        ProfileName::try_from(name.into()).map(Self)
    }

    /// Returns the profile's name.
    pub fn name(&self) -> &ProfileName {
        &self.0
    }
}

/// Why a profile directory cannot be located.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileRootError {
    /// `root_cache_path` is unset or blank.
    Missing,
    /// `root_cache_path` is not an absolute path.
    NotAbsolute,
}

/// Returns `<root>/profile-<name>`, joined onto the exact `root` string.
pub(crate) fn profile_dir(
    root: Option<&str>,
    name: &ProfileName,
) -> Result<PathBuf, ProfileRootError> {
    let root = root
        .filter(|r| !r.trim().is_empty())
        .ok_or(ProfileRootError::Missing)?;
    let root = Path::new(root);
    if !root.is_absolute() {
        return Err(ProfileRootError::NotAbsolute);
    }
    Ok(root.join(format!("profile-{}", name.as_str())))
}

/// Creates `dir` (and its parents) and checks that it is writable.
///
/// A failure to remove the write-check file is logged and does not fail the preparation.
///
/// On Unix the directory is created with mode `0o700`; on Windows it inherits
/// the ACL of the root.
pub(crate) fn prepare_profile_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(dir)?;
    let probe = dir.join(".bevy_cef_write_check");
    File::create(&probe)?;
    if let Err(error) = remove_file(&probe) {
        warn!(
            "could not remove the write-check file {}: {error}",
            probe.display()
        );
    }
    Ok(())
}

/// Prepares the profile context of each newly resolved profile webview and marks the webview
/// pending, or reports why it gets no browser.
fn request_profile_browsers(
    mut commands: Commands,
    mut browsers: NonSendMut<Browsers>,
    requester: Res<Requester>,
    root: Res<CefRootCachePath>,
    webviews: Query<(Entity, &WebviewProfile), Added<ResolvedWebviewUri>>,
) {
    for (webview, profile) in webviews.iter() {
        match prepare_profile(&mut browsers, &requester, root.0.as_deref(), profile.name()) {
            Ok(cache_path) => {
                commands.entity(webview).try_insert(PendingProfileBrowser {
                    cache_path,
                    navigation: None,
                });
            }
            Err(reason) => report_profile_error(&mut commands, webview, reason),
        }
    }
}

/// Requests a cookie-store flush on every profile context.
fn flush_profile_cookies(browsers: NonSend<Browsers>) {
    browsers.flush_profile_cookies();
}

/// Logs a warning for each webview that received a [`WebviewProfile`] after its browser was
/// already requested.
fn warn_late_profiles(webviews: Query<(Entity, Ref<ResolvedWebviewUri>), Added<WebviewProfile>>) {
    for webview in late_profile_webviews(&webviews) {
        warn!(
            "{webview} received a WebviewProfile after its browser was requested; the profile is ignored and the webview keeps running in memory"
        );
    }
}

/// Returns the webviews in `webviews` whose [`ResolvedWebviewUri`] was added before their
/// [`WebviewProfile`].
fn late_profile_webviews(
    webviews: &Query<(Entity, Ref<ResolvedWebviewUri>), Added<WebviewProfile>>,
) -> Vec<Entity> {
    webviews
        .iter()
        .filter(|(_, uri)| !uri.is_added())
        .map(|(webview, _)| webview)
        .collect()
}

fn added_profile_webview(
    webviews: Query<(), (Added<ResolvedWebviewUri>, With<WebviewProfile>)>,
) -> bool {
    !webviews.is_empty()
}

/// Creates the browser of every waiting profile webview whose context is ready, from the
/// webview's components as they are now.
///
/// A webview whose `WebviewSource` changed after its last resolution keeps waiting until the
/// change is resolved.
fn create_ready_profile_browsers(
    mut commands: Commands,
    mut browsers: NonSendMut<Browsers>,
    creation: BrowserCreation,
    focused: Res<FocusedWebview>,
    ticks: SystemChangeTick,
    webviews: Query<(
        Entity,
        &PendingProfileBrowser,
        Ref<ResolvedWebviewUri>,
        Ref<WebviewSource>,
        &WebviewSize,
        &WebviewDpr,
        &PreloadScripts,
        Option<&ZoomLevel>,
        Option<&AudioMuted>,
        Option<&HostWindow>,
    )>,
) {
    for (webview, pending, resolved, source, size, dpr, preload, zoom, muted, host_window) in
        webviews.iter()
    {
        if !browsers.is_profile_ready(&pending.cache_path) {
            continue;
        }
        let Some(uri) = initial_url(
            &resolved.0,
            resolved.last_changed(),
            pending.navigation.as_ref(),
            source.last_changed(),
            ticks.this_run(),
        ) else {
            continue;
        };
        let args = CreateArgs {
            uri,
            size: size.0,
            dpr: dpr.0,
            preload: &preload.0,
            zoom: zoom.map_or(0.0, |zoom| zoom.0),
            host_window: host_window.map(|window| window.0),
        };
        let created = creation.create(&mut browsers, webview, args, Some(&pending.cache_path));
        commands
            .entity(webview)
            .try_remove::<PendingProfileBrowser>();
        match created {
            Ok(()) => {
                if muted.is_some_and(|muted| muted.0) {
                    browsers.set_audio_muted(&webview, true);
                }
                commands
                    .entity(webview)
                    .try_insert(MuteAfterFirstCommit::default());
                if focused.0 == Some(webview) {
                    browsers.set_focus(&webview, true);
                }
            }
            Err(error) => {
                warn!(
                    "bevy_cef: failed to create the browser of profile webview {webview}: {error}"
                );
                report_profile_error(
                    &mut commands,
                    webview,
                    WebviewProfileErrorReason::BrowserCreation,
                );
            }
        }
    }
}

/// Records the first main-frame commit of a profile webview that still has to re-apply its mute.
fn note_first_commit(changed: On<AddressChanged>, mut webviews: Query<&mut MuteAfterFirstCommit>) {
    if let Ok(mut reapply) = webviews.get_mut(changed.webview) {
        reapply.committed = true;
    }
}

/// Sends the current [`AudioMuted`] of each profile webview whose first main-frame commit has
/// happened, then stops tracking it. A webview without `AudioMuted` is only no longer tracked.
fn reapply_mute_after_first_commit(
    mut commands: Commands,
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &MuteAfterFirstCommit, Option<&AudioMuted>)>,
) {
    for (webview, reapply, muted) in webviews.iter() {
        if reapply.committed {
            if let Some(muted) = muted {
                browsers.set_audio_muted(&webview, muted.0);
            }
            commands
                .entity(webview)
                .try_remove::<MuteAfterFirstCommit>();
        }
    }
}

/// Validates `name` and gets or creates its context, preparing its directory under `root` when
/// no context is registered for it yet.
///
/// Returns the profile's cache path, which keys its context in [`Browsers`].
fn prepare_profile(
    browsers: &mut Browsers,
    requester: &Requester,
    root: Option<&str>,
    name: &ProfileName,
) -> Result<String, WebviewProfileErrorReason> {
    name.revalidate()
        .map_err(|error| WebviewProfileErrorReason::InvalidName(error.to_string()))?;
    let dir = profile_dir(root, name).map_err(|_| WebviewProfileErrorReason::NoRoot)?;
    let cache_path = dir.to_string_lossy().into_owned();
    if browsers.has_profile(&cache_path) {
        return Ok(cache_path);
    }
    prepare_profile_dir(&dir)
        .map_err(|error| WebviewProfileErrorReason::Directory(error.to_string()))?;
    browsers
        .prepare_profile(&cache_path, requester.clone())
        .map_err(|error| match error {
            ProfileContextError::CreationFailed => WebviewProfileErrorReason::ContextCreation,
            ProfileContextError::DidNotLoad => WebviewProfileErrorReason::DidNotLoad,
        })?;
    Ok(cache_path)
}

fn report_profile_error(
    commands: &mut Commands,
    webview: Entity,
    reason: WebviewProfileErrorReason,
) {
    warn!("bevy_cef: profile webview {webview} gets no browser: {reason:?}");
    commands.trigger(WebviewProfileError { webview, reason });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::{NavigationPlugin, RequestNavigate};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Resource, Default)]
    struct ReportedErrors(Vec<WebviewProfileError>);

    #[derive(Resource, Default)]
    struct LateProfiles(Vec<Entity>);

    fn name(s: &str) -> ProfileName {
        ProfileName::try_from(s.to_owned()).unwrap()
    }

    fn profile_app(root: Option<String>) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            ProfilePlugin {
                cookie_flush_interval: Duration::from_secs(5),
            },
            NavigationPlugin,
        ))
        .insert_non_send(Browsers::default())
        .insert_resource(Requester(async_channel::unbounded().0))
        .insert_resource(CefRootCachePath(root))
        .init_resource::<ReportedErrors>()
        .add_observer(
            |error: On<WebviewProfileError>, mut reported: ResMut<ReportedErrors>| {
                reported.0.push(error.event().clone());
            },
        );
        app
    }

    fn spawn_profile_webview(app: &mut App, profile: WebviewProfile) -> Entity {
        app.world_mut()
            .spawn((
                WebviewSource::new("https://a.example"),
                ResolvedWebviewUri("https://a.example".into()),
                profile,
            ))
            .id()
    }

    fn reported_reasons(app: &App, webview: Entity) -> Vec<WebviewProfileErrorReason> {
        app.world()
            .resource::<ReportedErrors>()
            .0
            .iter()
            .filter(|error| error.webview == webview)
            .map(|error| error.reason.clone())
            .collect()
    }

    fn unique_temp_path() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "bevy_cef_profile_test_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn accepts_lowercase_digits_dash_underscore() {
        assert!(ProfileName::try_from("work_2-a".to_owned()).is_ok());
        assert!(ProfileName::try_from("a".repeat(64)).is_ok());
    }

    #[test]
    fn rejects_invalid_names() {
        let parse = |s: &str| ProfileName::try_from(s.to_owned());
        assert_eq!(parse(""), Err(ProfileNameError::Empty));
        assert_eq!(
            parse(&"a".repeat(65)),
            Err(ProfileNameError::TooLong { len: 65 })
        );
        assert_eq!(
            parse("Work"),
            Err(ProfileNameError::InvalidChar { ch: 'W' })
        );
        for bad in ["a/b", "a\\b", "..", "a b", "é"] {
            assert!(
                matches!(parse(bad), Err(ProfileNameError::InvalidChar { .. })),
                "{bad:?} must be rejected"
            );
        }
    }

    #[test]
    fn revalidate_accepts_a_valid_name() {
        assert_eq!(name("work").revalidate(), Ok(()));
    }

    #[test]
    fn deserializing_an_invalid_name_fails() {
        assert!(serde_json::from_str::<WebviewProfile>("\"Bad\"").is_err());
        let ok = serde_json::from_str::<WebviewProfile>("\"ok\"").unwrap();
        assert_eq!(serde_json::to_string(&ok).unwrap(), "\"ok\"");
    }

    #[test]
    fn profile_dir_joins_the_exact_root() {
        let root = std::env::temp_dir().join("cef_root");
        let root = root.to_str().unwrap();
        let expected = Path::new(root).join("profile-work");
        assert_eq!(profile_dir(Some(root), &name("work")), Ok(expected.clone()));
        let with_sep = format!("{root}{}", std::path::MAIN_SEPARATOR);
        assert_eq!(profile_dir(Some(&with_sep), &name("work")), Ok(expected));
    }

    #[test]
    fn profile_dir_requires_an_absolute_root() {
        let n = name("work");
        for missing in [None, Some(""), Some("   ")] {
            assert_eq!(profile_dir(missing, &n), Err(ProfileRootError::Missing));
        }
        assert_eq!(
            profile_dir(Some("relative/root"), &n),
            Err(ProfileRootError::NotAbsolute)
        );
    }

    #[test]
    fn prepare_profile_dir_creates_a_writable_dir() {
        let dir = unique_temp_path();
        prepare_profile_dir(&dir).unwrap();
        assert!(dir.is_dir());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&dir).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o700);
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn prepare_profile_dir_rejects_a_file_in_the_way() {
        let path = unique_temp_path();
        std::fs::write(&path, b"x").unwrap();
        assert!(prepare_profile_dir(&path).is_err());
        std::fs::remove_file(&path).unwrap();
    }

    /// Asserts that a profile name that fails revalidation reports `InvalidName` and leaves
    /// the webview without a pending browser.
    ///
    /// Case: a remote BRP client writes `"Bad"` into a webview's `WebviewProfile` through
    /// reflection, which bypasses the name check that construction and deserialization run.
    #[test]
    fn an_invalid_name_reports_invalid_name() {
        let root = unique_temp_path();
        let mut app = profile_app(Some(root.to_str().unwrap().to_owned()));
        let webview =
            spawn_profile_webview(&mut app, WebviewProfile(ProfileName::unchecked("Bad")));

        app.update();

        let reasons = reported_reasons(&app, webview);
        assert_eq!(reasons.len(), 1);
        assert!(matches!(
            reasons[0],
            WebviewProfileErrorReason::InvalidName(_)
        ));
        assert!(app.world().get::<PendingProfileBrowser>(webview).is_none());
        assert!(!root.exists());
    }

    /// Asserts that a profile webview reports `NoRoot` when `root_cache_path` is unset.
    ///
    /// Case: an app adds `CefPlugin::default()`, which leaves `root_cache_path` empty, and
    /// spawns a webview with the profile `work`.
    #[test]
    fn a_missing_root_reports_no_root() {
        let mut app = profile_app(None);
        let webview = spawn_profile_webview(&mut app, WebviewProfile::new("work").unwrap());

        app.update();

        assert_eq!(
            reported_reasons(&app, webview),
            vec![WebviewProfileErrorReason::NoRoot]
        );
        assert!(app.world().get::<PendingProfileBrowser>(webview).is_none());
    }

    /// Asserts that a file sitting where the profile directory belongs reports a `Directory`
    /// error and leaves the webview without a pending browser.
    ///
    /// Case: something wrote a plain file named `profile-work` into the root cache directory
    /// before the app spawns a webview with the profile `work`.
    #[test]
    fn a_file_at_the_profile_path_reports_a_directory_error() {
        let root = unique_temp_path();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("profile-work"), b"x").unwrap();
        let mut app = profile_app(Some(root.to_str().unwrap().to_owned()));
        let webview = spawn_profile_webview(&mut app, WebviewProfile::new("work").unwrap());

        app.update();

        let reasons = reported_reasons(&app, webview);
        assert_eq!(reasons.len(), 1);
        assert!(matches!(
            reasons[0],
            WebviewProfileErrorReason::Directory(_)
        ));
        assert!(app.world().get::<PendingProfileBrowser>(webview).is_none());
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// Asserts that a profile inserted a frame after the webview's URL was resolved is detected
    /// as late, while a profile added together with the URL is not.
    ///
    /// Case: an app spawns one webview with its profile in the same bundle and another that
    /// gets its profile inserted only after the first frame has run.
    #[test]
    fn a_profile_inserted_after_the_uri_is_detected_as_late() {
        let mut app = profile_app(None);
        app.init_resource::<LateProfiles>().add_systems(
            Update,
            |webviews: Query<(Entity, Ref<ResolvedWebviewUri>), Added<WebviewProfile>>,
             mut late: ResMut<LateProfiles>| {
                late.0.extend(late_profile_webviews(&webviews));
            },
        );
        let together = spawn_profile_webview(&mut app, WebviewProfile::new("work").unwrap());
        let later = app
            .world_mut()
            .spawn((
                WebviewSource::new("https://a.example"),
                ResolvedWebviewUri("https://a.example".into()),
            ))
            .id();

        app.update();
        assert!(app.world().resource::<LateProfiles>().0.is_empty());

        app.world_mut()
            .entity_mut(later)
            .insert(WebviewProfile::new("work").unwrap());
        app.update();

        let late = &app.world().resource::<LateProfiles>().0;
        assert_eq!(late, &vec![later]);
        assert!(!late.contains(&together));
    }

    /// Asserts that a `RequestNavigate` for a webview still waiting for its profile is stored
    /// as the URL to create the browser with.
    ///
    /// Case: an app spawns a profile webview and, before the profile's context has finished
    /// initializing, asks it to navigate to another page.
    #[test]
    fn request_navigate_while_pending_is_kept() {
        let mut app = profile_app(None);
        let webview = app
            .world_mut()
            .spawn(PendingProfileBrowser {
                cache_path: "x".into(),
                navigation: None,
            })
            .id();

        app.world_mut().trigger(RequestNavigate {
            webview,
            url: "https://b.example".into(),
        });

        let pending = app.world().get::<PendingProfileBrowser>(webview).unwrap();
        assert_eq!(
            pending.navigation.as_ref().map(|(url, _)| url.as_str()),
            Some("https://b.example")
        );
    }

    /// Asserts that `initial_url` picks whichever of the source change and the stored
    /// navigation happened last, and the resolved source when nothing navigated.
    ///
    /// Case: while a profile webview waits for its context, the app both sends a
    /// `RequestNavigate` and changes the `WebviewSource`, in either order.
    #[test]
    fn a_source_change_after_request_navigate_wins() {
        let this_run = Tick::new(100);
        let resolved_changed = Tick::new(60);
        let navigation = ("https://nav.example".to_owned(), Tick::new(40));
        assert_eq!(
            initial_url(
                "https://source.example",
                resolved_changed,
                Some(&navigation),
                Tick::new(50),
                this_run
            ),
            Some("https://source.example")
        );

        let navigation = ("https://nav.example".to_owned(), Tick::new(50));
        assert_eq!(
            initial_url(
                "https://source.example",
                resolved_changed,
                Some(&navigation),
                Tick::new(40),
                this_run
            ),
            Some("https://nav.example")
        );

        assert_eq!(
            initial_url(
                "https://source.example",
                resolved_changed,
                None,
                Tick::new(40),
                this_run
            ),
            Some("https://source.example")
        );
    }

    /// Asserts that `initial_url` returns nothing while the latest `WebviewSource` change is
    /// newer than its resolved URL, whether or not a navigation is stored.
    ///
    /// Case: while a profile webview waits for its context, the app changes the
    /// `WebviewSource` in `Update`, and the context reports ready later in that frame, before
    /// the next `PreUpdate` resolves the new source.
    #[test]
    fn an_unresolved_source_change_defers_creation() {
        let this_run = Tick::new(100);
        let resolved_changed = Tick::new(30);
        let source_changed = Tick::new(40);
        let navigation = ("https://nav.example".to_owned(), Tick::new(50));
        assert_eq!(
            initial_url(
                "https://old.example",
                resolved_changed,
                Some(&navigation),
                source_changed,
                this_run
            ),
            None
        );
        assert_eq!(
            initial_url(
                "https://old.example",
                resolved_changed,
                None,
                source_changed,
                this_run
            ),
            None
        );
    }

    /// Asserts that a profile webview keeps waiting to re-apply its mute until its first
    /// main-frame commit, and stops tracking it after that commit.
    ///
    /// Case: a profile webview muted while it waited for its context gets its browser, and its
    /// first page then commits, which resets the browser's mute in Chromium.
    #[test]
    fn the_mute_is_reapplied_once_at_the_first_commit() {
        let mut app = profile_app(None);
        let webview = app
            .world_mut()
            .spawn((AudioMuted(true), MuteAfterFirstCommit::default()))
            .id();

        app.update();
        assert!(app.world().get::<MuteAfterFirstCommit>(webview).is_some());

        commit_main_frame(&mut app, webview);
        assert!(app.world().get::<MuteAfterFirstCommit>(webview).is_none());
    }

    /// Asserts that the first main-frame commit stops tracking a profile webview's mute even
    /// when the webview no longer has an `AudioMuted`.
    ///
    /// Case: an app removes `AudioMuted` from a profile webview after its browser is created and
    /// before its first page commits.
    #[test]
    fn the_first_commit_clears_the_marker_without_audio_muted() {
        let mut app = profile_app(None);
        let webview = app.world_mut().spawn(MuteAfterFirstCommit::default()).id();

        commit_main_frame(&mut app, webview);
        assert!(app.world().get::<MuteAfterFirstCommit>(webview).is_none());
    }

    fn commit_main_frame(app: &mut App, webview: Entity) {
        app.world_mut().trigger(AddressChanged {
            webview,
            url: "https://a.example".into(),
            can_go_back: false,
            can_go_forward: false,
        });
        app.update();
    }
}
