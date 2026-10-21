//! The error type bevy_cef_core reports, and the result alias built on it.

use thiserror::Error;

/// A `Result` whose error is [`CefCoreError`].
pub type CefCoreResult<T = ()> = Result<T, CefCoreError>;

/// Every failure bevy_cef_core reports.
#[derive(Debug, Error)]
pub enum CefCoreError {
    /// A browser that could not be created.
    #[error(transparent)]
    CreateBrowser(#[from] CreateBrowserError),
    /// A profile's request context that could not be prepared.
    #[error(transparent)]
    ProfileContext(#[from] ProfileContextError),
}

/// The reason [`Browsers::create_browser`] did not create a browser.
///
/// [`Browsers::create_browser`]: crate::prelude::Browsers::create_browser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CreateBrowserError {
    /// CEF returned no browser, or a browser without a host.
    #[error("CEF did not create the browser")]
    NullBrowser,
    /// No context was prepared for the requested profile with [`Browsers::prepare_profile`].
    ///
    /// [`Browsers::prepare_profile`]: crate::prelude::Browsers::prepare_profile
    #[error("no request context was prepared for the profile")]
    UnknownProfile,
}

/// The reason a profile's request context could not be prepared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ProfileContextError {
    /// CEF returned no request context.
    #[error("CEF did not create the profile's request context")]
    CreationFailed,
    /// CEF rejected the cache path and created an in-memory context instead.
    #[error("CEF rejected the profile's cache path and fell back to in-memory storage")]
    DidNotLoad,
}
