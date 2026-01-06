use nih_plug::prelude::*;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::f32::consts::PI;
use std::sync::Arc;

struct Delay {
    params: Arc<DelayParams>,
    buffer: AllocRingBuffer<f32>,
}

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
            buffer: AllocRingBuffer::new(1),
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
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(0))
            .with_unit(" ms"),

            feedback: FloatParam::new("Feedback", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_value_to_string(formatters::v2s_f32_rounded(2)),

            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for Delay {
    const NAME: &'static str = "Delay";
    const VENDOR: &'static str = "Zhijian Li";
    const URL: &'static str = "https://codeberg.org/lzj15";
    const EMAIL: &'static str = "lzj15@proton.me";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];
    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.buffer = AllocRingBuffer::new(buffer_config.sample_rate as usize * 2);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let time = self.params.time.smoothed.next();
            let feedback = self.params.feedback.smoothed.next();
            let mix = self.params.mix.smoothed.next();

            let delay_position = time * 0.001 * context.transport().sample_rate;
            let floor = delay_position.floor();
            let fraction = delay_position - floor;

            let delay = self.buffer.get_signed(-floor as isize).unwrap_or(&0.0) * (1.0 - fraction)
                + self.buffer.get_signed(-floor as isize - 1).unwrap_or(&0.0) * fraction;
            let dry = *channel_samples.get_mut(0).unwrap();
            self.buffer.enqueue(dry + delay * feedback);

            let output = dry * (mix * 0.5 * PI).cos() + delay * (mix * 0.5 * PI).sin();
            *channel_samples.get_mut(0).unwrap() = output;
            *channel_samples.get_mut(1).unwrap() = output;
        }
        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for Delay {
    const CLAP_ID: &'static str = "org.codeberg.lzj15.delay-nih-plug";
    const CLAP_DESCRIPTION: Option<&'static str> = None;
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect];
}

impl Vst3Plugin for Delay {
    const VST3_CLASS_ID: [u8; 16] = *b"delay-nih-plug00";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_clap!(Delay);
nih_export_vst3!(Delay);
