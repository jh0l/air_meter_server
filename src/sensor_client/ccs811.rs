use embedded_ccs811::{mode, prelude::*, Ccs811Awake, MeasurementMode, ModeChangeError, SlaveAddr};
use linux_embedded_hal::I2cdev;
use nb::block;
use std::time::SystemTime;

pub struct Sensor {
    app: Ccs811Awake<I2cdev, mode::App>,
    start_time: u64,
    increment: MeasurementMode,
}

pub struct Reading {
    pub eco2: u16,
    pub evtoc: u16,
    pub read_time: u64,
    pub start_time: u64,
    pub increment: String,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
impl Sensor {
    pub fn new(mode: MeasurementMode) -> Result<Sensor, ()> {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let address = SlaveAddr::default();
        let sensor = Ccs811Awake::new(dev, address);
        match sensor.start_application() {
            Err(ModeChangeError { dev: _, error }) => {
                println!("Error during application start: {:?}", error);
                Err(())
            }
            Ok(mut sensor) => match sensor.set_mode(mode) {
                Err(err) => {
                    println!("{:?}", err);
                    Err(())
                }
                Ok(_) => Ok(Sensor {
                    app: sensor,
                    start_time: now(),
                    increment: mode,
                }),
            },
        }
    }

    pub fn new_1s() -> Result<Sensor, ()> {
        Sensor::new(MeasurementMode::ConstantPower1s)
    }

    fn mode_to_str(&self) -> String {
        use MeasurementMode::*;
        let r = match self.increment {
            Idle => "Idle",
            ConstantPower250ms => "ConstantPower250ms",
            ConstantPower1s => "ConstantPower1s",
            PulseHeating10s => "PulseHeating10s",
            LowPowerPulseHeating60s => "LowPowerPulseHeating60s",
        };
        r.to_owned()
    }

    pub fn read(
        &mut self,
    ) -> Result<
        Reading,
        embedded_ccs811::ErrorAwake<linux_embedded_hal::i2cdev::linux::LinuxI2CError>,
    > {
        let data = block!(self.app.data())?;
        Ok(Reading {
            eco2: data.eco2,
            evtoc: data.etvoc,
            increment: self.mode_to_str(),
            read_time: now(),
            start_time: self.start_time,
        })
    }
}
