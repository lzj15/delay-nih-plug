use nih_plug::prelude::*;
use std::sync::Arc;

struct Delay {
    params: Arc<DelayParams>,
    deque: std::collections::VecDeque<f32>,
}

/// The [`Params`] derive macro gathers all of the information needed for the wrapper to know about
/// the plugin's parameters, persistent serializable fields, and nested parameter groups. You can
/// also easily implement [`Params`] by hand if you want to, for instance, have multiple instances
/// of a parameters struct for multiple identical oscillators/filters/envelopes.
#[derive(Params)]
struct DelayParams {
    #[id = "time"]
    pub time: FloatParam,

    #[id = "feedback"]
    pub feedback: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayParams::default()),
            deque: std::collections::VecDeque::new(),
        }
    }
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            time: FloatParam::new(
                "Time",
                300.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 2000.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(0))
            .with_unit(" ms"),

            feedback: FloatParam::new("Feedback", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for Delay {
    const NAME: &'static str = "Delay";
    const VENDOR: &'static str = "lzj15";
    // You can use `env!("CARGO_PKG_HOMEPAGE")` to reference the homepage field from the
    // `Cargo.toml` file here
    const URL: &'static str = "https://codeberg.org/lzj15";
    const EMAIL: &'static str = "lzj15@proton.me";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // Setting this to `true` will tell the wrapper to split the buffer up into smaller blocks
    // whenever there are inter-buffer parameter changes. This way no changes to the plugin are
    // required to support sample accurate automation and the wrapper handles all of the boring
    // stuff like making sure transport and other timing information stays consistent between the
    // splits.
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    // This plugin doesn't need any special initialization, but if you need to do anything expensive
    // then this would be the place. State is kept around when the host reconfigures the
    // plugin. If we do need special initialization, we could implement the `initialize()` and/or
    // `reset()` methods

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.deque =
            std::collections::VecDeque::with_capacity((2.0 * buffer_config.sample_rate) as usize);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let time = self.params.time.value();
        let feedback = self.params.feedback.value();
        let mix = self.params.mix.value();

        // Calculate the index of the sample before time interval specified
        // -1.0 is there because the most recent sample in the queue has index 0
        let index = (-1.0 + time * 0.001 * context.transport().sample_rate) as usize;

        for mut channel_samples in buffer.iter_samples() {
            let delay = *self.deque.get(index).unwrap_or(&0.0);
            // Remove the last sample in the back of the queue to make room for next push_front()
            self.deque
                .remove((-1.0 + 2.0 * context.transport().sample_rate) as usize);
            self.deque
                .push_front(*channel_samples.get_mut(0).unwrap() + delay * feedback);
            *channel_samples.get_mut(0).unwrap() =
                *channel_samples.get_mut(0).unwrap() * (1.0 - mix) + delay * mix;
            *channel_samples.get_mut(1).unwrap() = *channel_samples.get_mut(0).unwrap();
        }
        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

impl ClapPlugin for Delay {
    const CLAP_ID: &'static str = "lzj15.delay-nih-plug";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("A very simple delay effect audio plugin using the nih-plug framework");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect];
}

impl Vst3Plugin for Delay {
    const VST3_CLASS_ID: [u8; 16] = *b"delay-nih-plug  ";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_clap!(Delay);
nih_export_vst3!(Delay);
