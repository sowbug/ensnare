// Copyright (c) 2023 Mike Tsao. All rights reserved.

//! This module assembles all the available entities so that an application can
//! use them.

use ensnare::prelude::*;

/// A wrapper that contains all the entities we know about.
pub struct MiniDawEntities {}
impl MiniDawEntities {
    /// Registers all the entities in this collection.
    pub fn register(factory: EntityFactory<dyn Entity>) -> EntityFactory<dyn Entity> {
        BuiltInEntities::register(factory)
    }
}
