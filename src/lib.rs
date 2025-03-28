#![warn(clippy::all, rust_2018_idioms)]
#![allow(refining_impl_trait)]

mod app;
pub mod emulator;
pub mod panes;
pub mod turing;

pub use app::TemplateApp;
