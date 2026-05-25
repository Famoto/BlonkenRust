use anyhow::Result;
use esp_idf_hal::adc::attenuation::DB_12;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::{ADC1, ADCCH0, ADCU1};
use esp_idf_hal::gpio::Gpio0;

pub const LIGHT_SENSOR_GPIO: u8 = 0;

const ADC_MAX: u32 = 4095;
const SAMPLES_PER_TICK: u32 = 4;
const FILTER_FRACTION_BITS: u32 = 8;
// At a 10 ms frame interval, 1/32 filtering settles smoothly over roughly 0.3 seconds.
const FILTER_SHIFT: u32 = 5;
// With the global LED current fixed at 15, this is close to the old level-1 minimum.
const MIN_INTENSITY: u8 = 17;
const MAX_INTENSITY: u8 = 255;

type SensorChannel = AdcChannelDriver<'static, ADCCH0<ADCU1>, AdcDriver<'static, ADCU1>>;

pub struct LightReading {
    pub filtered_raw: u16,
    pub intensity: u8,
}

pub struct LightSensor {
    channel: SensorChannel,
    filtered_raw_scaled: u32,
}

impl LightSensor {
    pub fn new(adc: ADC1<'static>, pin: Gpio0<'static>) -> Result<Self> {
        let adc = AdcDriver::new(adc)?;
        let config = AdcChannelConfig {
            attenuation: DB_12,
            ..Default::default()
        };
        let mut channel = AdcChannelDriver::new(adc, pin, &config)?;
        let filtered_raw_scaled = u32::from(average_samples(&mut channel)?) << FILTER_FRACTION_BITS;

        Ok(Self {
            channel,
            filtered_raw_scaled,
        })
    }

    pub fn sample(&mut self) -> Result<LightReading> {
        let sample_scaled = u32::from(average_samples(&mut self.channel)?) << FILTER_FRACTION_BITS;
        self.filtered_raw_scaled = smooth(self.filtered_raw_scaled, sample_scaled);
        let filtered_raw = ((self.filtered_raw_scaled + (1 << (FILTER_FRACTION_BITS - 1)))
            >> FILTER_FRACTION_BITS) as u16;

        Ok(LightReading {
            filtered_raw,
            intensity: intensity_from_light(filtered_raw),
        })
    }
}

fn average_samples(channel: &mut SensorChannel) -> Result<u16> {
    let mut sum = 0u32;

    for _ in 0..SAMPLES_PER_TICK {
        sum += u32::from(channel.read_raw()?);
    }

    Ok((sum / SAMPLES_PER_TICK) as u16)
}

fn smooth(previous: u32, sample: u32) -> u32 {
    if sample >= previous {
        previous + ((sample - previous) >> FILTER_SHIFT)
    } else {
        previous - ((previous - sample) >> FILTER_SHIFT)
    }
}

fn intensity_from_light(raw: u16) -> u8 {
    let raw = u32::from(raw).min(ADC_MAX);
    let range = u32::from(MAX_INTENSITY - MIN_INTENSITY);
    MIN_INTENSITY + (raw * range / ADC_MAX) as u8
}
