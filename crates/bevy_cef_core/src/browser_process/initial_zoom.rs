//! Holds a webview's requested zoom until its main frame commits.
//!
//! Chromium ignores a zoom set before the main frame has a live renderer and a
//! committed URL, so the level is held and applied once at the first commit.

use std::cell::Cell;
use std::rc::Rc;

/// A zoom state shared between a webview's browser entry and its load handler.
pub type SharedInitialZoom = Rc<Cell<InitialZoom>>;

/// Where a webview is on the way to its first main-frame commit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InitialZoom {
    /// The main frame has not committed yet; holds the level to apply at commit, if any.
    Waiting(Option<f64>),
    /// The main frame has committed; zoom changes are sent immediately.
    Committed,
}

/// What to do with a zoom request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomRequest {
    /// The level was stored for the first commit.
    Held,
    /// The level must be sent to CEF now.
    SendNow,
}

impl InitialZoom {
    /// Starts waiting with `level`; the default level `0.0` holds nothing.
    pub fn waiting(level: f64) -> Self {
        Self::Waiting(Self::held(level))
    }

    /// Stores `level` while waiting, or asks for it to be sent once committed.
    pub fn request(&mut self, level: f64) -> ZoomRequest {
        match self {
            Self::Waiting(held) => {
                *held = Self::held(level);
                ZoomRequest::Held
            }
            Self::Committed => ZoomRequest::SendNow,
        }
    }

    /// Returns the held level once and moves to [`InitialZoom::Committed`].
    pub fn commit(&mut self) -> Option<f64> {
        match std::mem::replace(self, Self::Committed) {
            Self::Waiting(held) => held,
            Self::Committed => None,
        }
    }

    fn held(level: f64) -> Option<f64> {
        (level != 0.0).then_some(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that a webview left at the default zoom applies nothing at commit.
    ///
    /// Case: a webview is spawned without a zoom change and loads its page.
    #[test]
    fn a_default_zoom_applies_nothing_at_commit() {
        assert_eq!(InitialZoom::waiting(0.0).commit(), None);
    }

    /// Asserts that a non-default zoom is applied at the first commit only.
    ///
    /// Case: a webview is spawned at 1.5 and later navigates, committing again.
    #[test]
    fn a_non_default_zoom_applies_once() {
        let mut zoom = InitialZoom::waiting(1.5);
        assert_eq!(zoom.commit(), Some(1.5));
        assert_eq!(zoom.commit(), None);
    }

    /// Asserts that a change before the commit is held and replaces the earlier level.
    ///
    /// Case: the app changes the zoom while the page is still loading.
    #[test]
    fn a_change_before_commit_replaces_the_held_level() {
        let mut zoom = InitialZoom::waiting(1.0);
        assert_eq!(zoom.request(2.0), ZoomRequest::Held);
        assert_eq!(zoom.commit(), Some(2.0));
    }

    /// Asserts that resetting to the default before the commit applies nothing.
    ///
    /// Case: the app zooms in and then resets the zoom while the page is still loading.
    #[test]
    fn resetting_to_default_before_commit_applies_nothing() {
        let mut zoom = InitialZoom::waiting(1.0);
        assert_eq!(zoom.request(0.0), ZoomRequest::Held);
        assert_eq!(zoom.commit(), None);
    }

    /// Asserts that a request after the commit is sent immediately.
    ///
    /// Case: the user zooms with the mouse wheel after the page has loaded.
    #[test]
    fn after_commit_changes_are_sent_now() {
        let mut zoom = InitialZoom::waiting(0.0);
        zoom.commit();
        assert_eq!(zoom.request(3.0), ZoomRequest::SendNow);
    }
}
