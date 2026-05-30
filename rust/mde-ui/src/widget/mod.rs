//! Win2000 Classic widgets for iced.
//!
//! The bevel model ([`bevel`]) is implemented and unit-tested. The iced
//! `Widget`/style wiring (3D button, sunken field, title bar, menubar, tree,
//! column list) lands as the components are built — see tasks for mde-ui.

pub mod bevel;

pub use bevel::Bevel;
