use nih_plug::prelude::*;
use std::sync::Arc;

struct Saturator {
    params: Arc<SaturatorParams>,
}

#[derive(Enum, Debug, PartialEq)]
enum Function {
    NaturalLog,
    Sigmoid,
}

#[derive(Params)]
struct SaturatorParams {
    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "function"]
    pub function: EnumParam<Function>,
}

impl Default for Saturator {
    fn default() -> Self {
        Self {
            params: Arc::new(SaturatorParams::default()),
        }
    }
}

impl Default for SaturatorParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            function: EnumParam::new("Function", Function::NaturalLog),
        }
    }
}

impl Plugin for Saturator {
    const NAME: &'static str = "Saturator";
    const VENDOR: &'static str = "Martin GRUDLER";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "devel@grudler.eu";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                match self.params.function.value() {
                    Function::NaturalLog => {
                        if sample.is_sign_positive() {
                            *sample = (f32::ln(*sample + 1.0)) / f32::ln(2.);
                        } else {
                            *sample = -((f32::ln(-*sample + 1.0)) / f32::ln(2.));
                        }
                    }
                    Function::Sigmoid => {
                        *sample = 2.0 * (1.0 / (1.0 + f32::exp(-5.0 * (*sample))) - 0.5);
                    }
                }

                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Saturator {
    const CLAP_ID: &'static str = "fr.tsgeek.saturator";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple saturator plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Distortion,
        ClapFeature::Mixing,
    ];
}

impl Vst3Plugin for Saturator {
    const VST3_CLASS_ID: [u8; 16] = *b"N6KtgTL2ZknCEU4y";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(Saturator);
nih_export_vst3!(Saturator);
