use actix::*;
use embedded_ccs811::{prelude::*, Ccs811Awake, MeasurementMode, ModeChangeError, SlaveAddr};
use linux_embedded_hal::I2cdev;
use nb::block;
use std::time::SystemTime;

use crate::sensor_client::{ConnectSession, CurrentMode, Message, Sensor, TakeReading};

pub struct Reading {
    pub eco2: u16,
    pub evtoc: u16,
    pub read_time: u64,
    pub start_time: u64,
    pub increment: String,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Actor for Sensor {
    type Context = SyncContext<Self>;
}

/// handle receiving connection to SessionClient
impl Handler<ConnectSession> for Sensor {
    type Result = ();

    fn handle(&mut self, msg: ConnectSession, _: &mut SyncContext<Self>) {
        msg.addr
            .try_send(CurrentMode {
                inc: self.increment,
            })
            .unwrap();
        self.session = Some(msg.addr);
    }
}

/// handle requests to take a reading
impl Handler<TakeReading> for Sensor {
    type Result = ();

    fn handle(&mut self, _: TakeReading, _: &mut SyncContext<Self>) {
        self.take_reading();
    }
}

/// handle messages from SessionClient (probably to change increment type)
impl Handler<Message> for Sensor {
    type Result = ();

    fn handle(&mut self, msg: Message, _: &mut SyncContext<Self>) {
        // TODO handle increment change request
        println!("SENSOR RECEIVED {:?}", msg);
        todo!("handle changing increment type for {:?}", msg);
    }
}

impl Sensor {
    pub fn new(mode: MeasurementMode) -> Result<Sensor, ()> {
        Sensor {
            app: None,
            start_time: now_secs(),
            increment: mode,
            session: None,
        }
        .load_sensor()
    }

    pub fn new_1s() -> Result<Sensor, ()> {
        Sensor::new(MeasurementMode::ConstantPower1s)
    }

    #[cfg(target_arch = "arm")]
    pub fn load_sensor(mut self) -> Result<Sensor, ()> {
        let dev = I2cdev::new("/dev/i2c-1").unwrap();
        let address = SlaveAddr::default();
        let sensor = Ccs811Awake::new(dev, address);
        match sensor.start_application() {
            Err(ModeChangeError { dev: _, error }) => {
                println!("Error during application start: {:?}", error);
                Err(())
            }
            Ok(mut sensor) => match sensor.set_mode(self.increment) {
                Err(err) => {
                    println!("{:?}", err);
                    Err(())
                }
                Ok(_) => {
                    self.app = Some(sensor);
                    Ok(self)
                }
            },
        }
    }

    #[cfg(not(target_arch = "arm"))]
    pub fn load_sensor(self) -> Result<Sensor, ()> {
        println!("<<SENSOR IN TEST MODE - NOT REAL READINGS>>");
        Ok(self)
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

    pub fn take_reading(&mut self) {
        // read() blocks the thread
        match &mut self.session.clone() {
            Some(session) => match self.read() {
                Ok(read) => {
                    let cmd = format!(
                                "{{ \"eco2\": {} \"evtoc\":{} \"increment\":{} \"read_time\":{} \"start_time\":{} }}",
                                read.eco2, read.evtoc, read.increment, read.read_time, read.start_time
                            );
                    session.do_send(Message(cmd));
                }
                Err(err) => {
                    println!("SENSOR READ ERROR: {:?}", err);
                }
            },
            None => {
                println!("Sensor waiting for session");
            }
        };
    }

    pub fn read(
        &mut self,
    ) -> Result<
        Reading,
        embedded_ccs811::ErrorAwake<linux_embedded_hal::i2cdev::linux::LinuxI2CError>,
    > {
        match &mut self.app {
            Some(app) => {
                let data = block!(app.data())?;
                Ok(Reading {
                    eco2: data.eco2,
                    evtoc: data.etvoc,
                    increment: self.mode_to_str(),
                    read_time: now_secs(),
                    start_time: self.start_time,
                })
            }
            None => Ok(Reading {
                eco2: now_secs() as u16,
                evtoc: now_secs() as u16 / 2,
                increment: self.mode_to_str(),
                read_time: now_secs(),
                start_time: self.start_time,
            }),
        }
    }
}
