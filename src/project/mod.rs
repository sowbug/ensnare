// Copyright (c) 2024 Mike Tsao. All rights reserved.

//! A full song: the composition, the arrangement of instruments, the instrument
//! and effect parameters, and so on.

/// The most commonly used imports.
pub mod prelude {
    pub use super::{ArrangementViewMode, Project, ProjectTitle, ProjectViewState};
}

pub use project::{ArrangementViewMode, Project, ProjectTitle, ProjectViewState};

mod project;
