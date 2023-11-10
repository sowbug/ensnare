// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;
use ensnare_proc_macros::{Control, Params};

#[derive(Debug, Default, Control, Params)]
pub struct Compressor {
    /// The level above which compression takes effect. Range is 0.0..=1.0, 0.0
    /// corresponds to quietest, and 1.0 corresponds to 0dB.
    #[control]
    #[params]
    threshold: Normal,

    /// How much to compress the audio above the threshold. For example, 2:1
    /// means that a 2dB input increase leads to a 1dB output increase. Note
    /// that this value is actually the inverted ratio, so that 2:1 is 0.5 (1
    /// divided by 2), and 1:4 is 0.25 (1 divided by 4). Thus, 1.0 means no
    /// compression, and 0.0 is infinite compression (the output remains a
    /// constant amplitude no matter what).
    #[control]
    #[params]
    ratio: ParameterType,

    /// How soon the compressor activates after the level exceeds the threshold.
    /// Time in seconds.
    #[control]
    #[params]
    attack: ParameterType,

    /// How soon the compressor deactivates after the level drops below the
    /// threshold. Time in seconds.
    #[control]
    #[params]
    release: ParameterType,

    // TODO
    #[allow(dead_code)]
    current_gain: f32,
}
impl Serializable for Compressor {}
impl TransformsAudio for Compressor {
    fn transform_channel(&mut self, _channel: usize, input_sample: Sample) -> Sample {
        let input_sample_positive = input_sample.0.abs();
        let threshold = self.threshold.0;
        if input_sample_positive > threshold {
            // TODO: this expression is (a + b - a) * c * d, which is just b * c
            // * d, which is clearly wrong. Fix it. (Too tired right now to look
            //   into how compression should work)
            Sample::from(
                (threshold + (input_sample_positive - threshold) * self.ratio)
                    * input_sample.0.signum(),
            )
        } else {
            input_sample
        }
    }
}
impl Configurable for Compressor {}
impl Compressor {
    pub fn new_with(params: &CompressorParams) -> Self {
        Self {
            threshold: params.threshold(),
            ratio: params.ratio(),
            attack: params.attack(),
            release: params.release(),
            ..Default::default()
        }
    }

    pub fn threshold(&self) -> Normal {
        self.threshold
    }

    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    pub fn attack(&self) -> f64 {
        self.attack
    }

    pub fn release(&self) -> f64 {
        self.release
    }

    pub fn set_threshold(&mut self, threshold: Normal) {
        self.threshold = threshold;
    }

    pub fn set_ratio(&mut self, ratio: ParameterType) {
        self.ratio = ratio;
    }

    pub fn set_attack(&mut self, attack: ParameterType) {
        self.attack = attack;
    }

    pub fn set_release(&mut self, release: ParameterType) {
        self.release = release;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_compressor() {
        const THRESHOLD: SampleType = 0.25;
        let mut fx = Compressor::new_with(&CompressorParams {
            threshold: Normal::from(THRESHOLD),
            ratio: 0.5,
            attack: 0.0,
            release: 0.0,
        });
        assert_eq!(
            fx.transform_channel(0, Sample::from(0.35)),
            Sample::from((0.35 - THRESHOLD) * 0.5 + THRESHOLD)
        );
    }

    #[test]
    fn nothing_compressor() {
        let mut fx = Compressor::new_with(&CompressorParams {
            threshold: Normal::from(0.25),
            ratio: 1.0,
            attack: 0.0,
            release: 0.0,
        });
        assert_eq!(
            fx.transform_channel(0, Sample::from(0.35f32)),
            Sample::from(0.35f32)
        );
    }

    #[test]
    fn infinite_compressor() {
        let mut fx = Compressor::new_with(&CompressorParams {
            threshold: Normal::from(0.25),
            ratio: 0.0,
            attack: 0.0,
            release: 0.0,
        });
        assert_eq!(
            fx.transform_channel(0, Sample::from(0.35)),
            Sample::from(0.25)
        );
    }
}