#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(type_alias_impl_trait)]
#![forbid(unsafe_code)]

use eeprom24x::{Eeprom24x, SlaveAddr};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    bind_interrupts,
    config::Config,
    i2c::{self, I2c},
    peripherals::I2C1,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

pub static I2C1_BUS: StaticCell<
    embassy_sync::mutex::Mutex<CriticalSectionRawMutex, I2c<'static, I2C1, i2c::Async>>,
> = StaticCell::new();

// Bind interrupts to the handlers inside embassy.
bind_interrupts!(struct Irqs {
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

pub const I2C1_FREQUENCY_HZ: u32 = 400_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Config::default());

    let i2c1_sda = p.PIN_18;
    let i2c1_scl = p.PIN_19;

    let mut i2c1_config = i2c::Config::default();
    i2c1_config.frequency = I2C1_FREQUENCY_HZ;

    let i2c1_bus = I2c::new_async(p.I2C1, i2c1_scl, i2c1_sda, Irqs, i2c1_config);

    let eeprom = Eeprom24x::new_24x08(i2c1_bus, SlaveAddr::Default);
    let i2c1_bus = eeprom.destroy(); // This works fine.

    let i2c1_bus = embassy_sync::mutex::Mutex::new(i2c1_bus);
    let i2c1_bus = I2C1_BUS.init(i2c1_bus);

    let dac_bus: I2cDevice<'static, CriticalSectionRawMutex, I2c<'static, I2C1, i2c::Async>> =
        I2cDevice::new(i2c1_bus);
    let eeprom_bus = I2cDevice::new(i2c1_bus);

    //let eeprom = Eeprom24x::new_24x08(eeprom_bus, SlaveAddr::Default); // Why doesn't this work then?

    loop {}
}
