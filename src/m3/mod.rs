//! Material Design 3 foundations for icedtea.
//!
//! See <https://m3.material.io/get-started>. Widgets paint from
//! [`crate::theme::Tokens`] seeded by [`Scheme`].

pub mod color;
pub mod density;
pub mod elevation;
pub mod mapping;
pub mod shape;
pub mod state;
pub mod type_scale;

pub use color::{scheme_dark, scheme_light, Scheme};
pub use density::{Density, DensityName, GRID};
pub use elevation::Elevation;
pub use shape::{Corner, Shape};
pub use state::ControlState;
pub use type_scale::{TypeRole, TypeScale};
