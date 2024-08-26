#![no_std]
mod errors;

const MAX_CHANNEL_VALUE: u16 = 26427;

#[derive(Debug)]
pub enum Channel {
    A0,
    A1,
    A2,
    A3,
}

pub enum Address {
    X48,
    X49,
    X4A,
    X4B,
}

pub enum Range {
    Within255,
    Within511,
    Within1023,
    Full,
}

impl Range {
    fn scale_to_range(value: u16, range: &Range, inverted: bool) -> u16 {
        match range {
            Range::Within255 => match inverted {
                true => 255 - u16::min(255, value / 103),
                false => u16::min(255, value / 103),
            },
            Range::Within511 => match inverted {
                true => 511 - u16::min(511, value / 51),
                false => u16::min(511, value / 51),
            },
            Range::Within1023 => match inverted {
                true => 1023 - u16::min(1023, value / 25),
                false => u16::min(1023, value / 25),
            },
            Range::Full => match inverted {
                true => u16::min(value, MAX_CHANNEL_VALUE),
                false => MAX_CHANNEL_VALUE - u16::min(value, MAX_CHANNEL_VALUE),
            },
        }
    }
}

impl Address {
    fn default() -> Address {
        Address::X48
    }

    fn from_u8(value: u8) -> Address {
        match value {
            0x48 => Address::X48,
            0x49 => Address::X49,
            0x4A => Address::X4A,
            0x4B => Address::X4B,
            _ => Address::X48,
        }
    }
}

pub struct Change {
    pub channel: Channel,
    pub value: u16,
}

pub struct Knobz<I2C> {
    pub channel_0: u16,
    pub channel_1: u16,
    pub channel_2: u16,
    pub channel_3: u16,
    channel_0_range: Range,
    channel_1_range: Range,
    channel_2_range: Range,
    channel_3_range: Range,
    channel_0_inverted: bool,
    channel_1_inverted: bool,
    channel_2_inverted: bool,
    channel_3_inverted: bool,
    ads1115: ads1x1x::Ads1x1x<
        I2C,
        ads1x1x::ic::Ads1115,
        ads1x1x::ic::Resolution16Bit,
        ads1x1x::mode::OneShot,
    >,
    channel_index: Channel,
    timer_us: u32,
}

impl<I2C, E> Knobz<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    pub fn new(i2c_device: I2C, address: Address) -> Result<Self, crate::errors::Error> {
        let ads1115_address = match address {
            Address::X48 => ads1x1x::SlaveAddr::default(),
            Address::X49 => ads1x1x::SlaveAddr::Vdd,
            Address::X4A => ads1x1x::SlaveAddr::Sda,
            Address::X4B => ads1x1x::SlaveAddr::Scl,
        };

        let timer_offset_us = match address {
            Address::X48 => 0,
            Address::X49 => 250,
            Address::X4A => 500,
            Address::X4B => 750,
        };

        let mut adc = ads1x1x::Ads1x1x::new_ads1115(i2c_device, ads1115_address);
        adc.set_full_scale_range(ads1x1x::FullScaleRange::Within4_096V)
            .map_err(|_| crate::errors::Error::I2C)?;
        Ok(Knobz {
            channel_0: 0,
            channel_1: 0,
            channel_2: 0,
            channel_3: 0,
            channel_0_range: Range::Within1023,
            channel_1_range: Range::Within1023,
            channel_2_range: Range::Within1023,
            channel_3_range: Range::Within1023,
            channel_0_inverted: false,
            channel_1_inverted: false,
            channel_2_inverted: false,
            channel_3_inverted: false,
            ads1115: adc,
            channel_index: Channel::A0,
            timer_us: timer_offset_us,
        })
    }

    pub fn destroy(self) -> I2C {
        self.ads1115.destroy_ads1115()
    }

    pub fn set_channel_range(&mut self, channel: Channel, range: Range) {
        match channel {
            Channel::A0 => self.channel_0_range = range,
            Channel::A1 => self.channel_1_range = range,
            Channel::A2 => self.channel_2_range = range,
            Channel::A3 => self.channel_3_range = range,
        }
    }

    pub fn set_invert_channel(&mut self, channel: Channel, inverted: bool) {
        match channel {
            Channel::A0 => self.channel_0_inverted = inverted,
            Channel::A1 => self.channel_1_inverted = inverted,
            Channel::A2 => self.channel_2_inverted = inverted,
            Channel::A3 => self.channel_3_inverted = inverted,
        }
    }

    pub fn update(&mut self, dt_us: u32) -> Option<Change> {
        self.timer_us += dt_us;
        if self.timer_us < 1000 {
            return None;
        }
        self.timer_us = 0;

        match self.channel_index {
            Channel::A0 => match self.ads1115.read(ads1x1x::channel::SingleA0) {
                Ok(pot_0) => {
                    let pot_0 = Range::scale_to_range(
                        i16::max(pot_0, 0) as u16,
                        &self.channel_0_range,
                        self.channel_0_inverted,
                    );
                    let changed = self.channel_0 != pot_0;
                    self.channel_0 = pot_0;
                    self.channel_index = Channel::A1;
                    if changed {
                        Some(Change {
                            channel: Channel::A0,
                            value: pot_0,
                        })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
            Channel::A1 => match self.ads1115.read(ads1x1x::channel::SingleA1) {
                Ok(pot_1) => {
                    let pot_1 = Range::scale_to_range(
                        i16::max(pot_1, 0) as u16,
                        &self.channel_1_range,
                        self.channel_1_inverted,
                    );
                    let changed = self.channel_1 != pot_1;
                    self.channel_1 = pot_1;
                    self.channel_index = Channel::A2;
                    if changed {
                        Some(Change {
                            channel: Channel::A1,
                            value: pot_1,
                        })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
            Channel::A2 => match self.ads1115.read(ads1x1x::channel::SingleA2) {
                Ok(pot_2) => {
                    let pot_2 = Range::scale_to_range(
                        i16::max(pot_2, 0) as u16,
                        &self.channel_2_range,
                        self.channel_2_inverted,
                    );
                    let changed = self.channel_2 != pot_2;
                    self.channel_2 = pot_2;
                    self.channel_index = Channel::A3;
                    if changed {
                        Some(Change {
                            channel: Channel::A2,
                            value: pot_2,
                        })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
            Channel::A3 => match self.ads1115.read(ads1x1x::channel::SingleA3) {
                Ok(pot_3) => {
                    let pot_3 = Range::scale_to_range(
                        i16::max(pot_3, 0) as u16,
                        &self.channel_3_range,
                        self.channel_3_inverted,
                    );
                    let changed = self.channel_3 != pot_3;
                    self.channel_3 = pot_3;
                    self.channel_index = Channel::A0;

                    if changed {
                        Some(Change {
                            channel: Channel::A3,
                            value: pot_3,
                        })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            },
        }
    }
}
