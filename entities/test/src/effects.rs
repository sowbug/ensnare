// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{InnerTransformsAudio, IsEntity, Metadata};
use serde::{Deserialize, Serialize};

/// The smallest possible [IsEntity].
#[derive(Debug, Default, IsEntity, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controllable,
    Controls,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner,
    Ticks,
    TransformsAudio
)]
pub struct TestEffect {
    uid: Uid,
}
impl TestEffect {
    pub fn new_with(uid: Uid) -> Self {
        Self { uid }
    }
}

/// Flips the sign of every audio sample it sees.
#[derive(Debug, Default, IsEntity, InnerTransformsAudio, Metadata, Serialize, Deserialize)]
#[entity(
    Configurable,
    Controllable,
    Controls,
    Displays,
    GeneratesStereoSample,
    HandlesMidi,
    Serializable,
    SkipInner,
    Ticks
)]
pub struct TestEffectNegatesInput {
    uid: Uid,
    inner: ensnare_cores::TestEffectNegatesInput,
}
impl TestEffectNegatesInput {
    pub fn new_with(uid: Uid) -> Self {
        Self {
            uid,
            inner: Default::default(),
        }
    }
}
