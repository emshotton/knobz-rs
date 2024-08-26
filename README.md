# KNOBZ

An embedded rust library for speaking to the KNOBZ chainable I2C potentiometers.

```rust

let mut knobz = Knobz::new(i2c, knobz::Address::X48).unwrap();
knobz.set_channel_range(knobz::Channel::A0, knobz::Range::Within255);

let mut previous_time_us = loop_timer.get_counter_low();
loop {
    let current_time_us = loop_timer.get_counter_low();
    let elapsed_time_us = current_time_us.wrapping_sub(previous_time_us);
    previous_time_us = current_time_us;

    match knobz.update(elapsed_time_us) {
        Some(value_change) => match value_change.channel {
            knobz::Channel::A0 => {
                info!("Knobz 1: Channel A0 Value {:?}", value_change.value);
            }
            knobz::Channel::A1 => {
                info!("Knobz 1: Channel A1 Value {:?}", value_change.value);
            }
            knobz::Channel::A2 => {
                info!("Knobz 1: Channel A2 Value {:?}", value_change.value);
            }
            knobz::Channel::A3 => {
                info!("Knobz 1: Channel A3 Value {:?}", value_change.value);
            }
        },
        None => {}
    };
}
```
