use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{DirBuilder, File, remove_file};
use std::path::{Path, PathBuf};

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
    #[cfg_attr(not(test), expect(dead_code, reason = "used by the profile systems"))]
    pub(crate) fn revalidate(&self) -> Result<(), ProfileNameError> {
        Self::validate(&self.0)
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
#[cfg_attr(not(test), expect(dead_code, reason = "used by the profile systems"))]
pub(crate) enum ProfileRootError {
    /// `root_cache_path` is unset or blank.
    Missing,
    /// `root_cache_path` is not an absolute path.
    NotAbsolute,
}

/// Returns `<root>/profile-<name>`, joined onto the exact `root` string.
#[cfg_attr(not(test), expect(dead_code, reason = "used by the profile systems"))]
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
/// On Unix the directory is created with mode `0o700`; on Windows it inherits
/// the ACL of the root.
#[cfg_attr(not(test), expect(dead_code, reason = "used by the profile systems"))]
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
    remove_file(&probe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn name(s: &str) -> ProfileName {
        ProfileName::try_from(s.to_owned()).unwrap()
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
}
