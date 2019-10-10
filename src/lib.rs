//! This driver provides Rust support for a number of Chinese-manufactured UHF RFID
//! Gen2 reader modules based on the Impinj Indy R2000 RF chipset with an AVR ARM processor. 
//!
//! It appears that these readers are based on a white-label module design which is used by a
//! number of Chinese manufacturers. The original designer remains unknown. I've named this module
//! "invelion" as this is the device I have.
//!
//! ## Example Code
//!
//! Examples of the use of this library can be found in the `examples` directory.
//!
//! ## Supported Readers
//!
//! Unless otherwise noted, the modules listed below are *not tested* with this library, but are
//! suspected to use the same protocol due to visual similarity, similarity to modules from the
//! same manufacturer, or availability of manuals (mostly on the FCC website) depicting the same
//! evaluation software.
//!
//! Please let me know if you get a new reader working with this code!
//!
//! [Invelion/INNOD](http://www.innod-rfid.net/) (Shenzhen Invelion Technology CO., Ltd):
//!   * IND905 / YR905 / IND901 / YR901 (*tested and working*)
//!   * YR900
//!   * IND904
//!   * IND9010 (suspected identical to Rodinbell D100)
//!   * IND9051
//!
//! [Rodinbell](http://www.rodinbell.com/) (Shenzhen Rodinbell Technology CO., Ltd):
//!   * D100 ([FCC](https://fcc.io/2AKQD-D100))
//!   * M500 ([FCC](https://fcc.io/2AKQD-M500))
//!   * M2800 ([FCC](https://fcc.io/2AKQD-M2800))
//!   * M2600 ([FCC](https://fcc.io/2AKQD-M2600))
//!   * M2900 ([FCC](https://fcc.io/2AKQD-M2900))
//!   * S-8600 ([FCC](https://fcc.io/2AKQD-S-8600A))
//!   * S-8800
//!

extern crate bitreader;
extern crate log;
extern crate num_enum;
extern crate serial;
extern crate failure;

pub mod error;
pub mod protocol;

use log::debug;
use serial::core::prelude::*;
use std::io::Read;
use std::iter;
use std::time::Duration;

use crate::error::Result;
use crate::protocol::{
    convert_from_frequency, Command, CommandType, InventoryItem,
    InventoryResult, Response, START_BYTE,
};

pub struct Reader {
    port: serial::SystemPort,
    antenna_count: usize,
    address: u8,
}

impl Reader {

    /// Create the object and connect to the serial port
    ///
    /// `port` should be the name of a serial port device.
    /// `address` is the address of the reader, which is usually 1.
    /// `antenna_count` is the number of antenna ports the reader has.
    pub fn new(port: &str, address: u8, antenna_count: u8) -> Result<Reader> {
        let mut port = serial::open(port)
            .map_err(|e| format!("Unable to connect to serial port {}: {:?}", port, e))?;
        port.reconfigure(&|settings| {
            try!(settings.set_baud_rate(serial::Baud115200));
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })
        .map_err(|e| format!("Failed to configure serial port: {}", e))?;

        port.set_timeout(Duration::from_millis(1000))
            .map_err(|e| format!("Failed to set serial port timeout: {}", e))?;
        Ok(Reader {
            port: port,
            address: address,
            antenna_count: antenna_count as usize,
        })
    }

    /// Send a command to the reader
    fn send(&mut self, cmd: Command) -> Result<()> {
        let cmd_bytes = cmd.to_bytes();
        debug!("Send {:?}: {:?}", cmd.command, cmd_bytes);
        std::io::Write::write(&mut self.port, &cmd_bytes)?;
        Ok(())
    }

    /// Wait for a start byte, discarding any other bytes received.
    ///
    /// This allows the driver to recover from unexpected timeouts - the timeout error still needs
    /// to be caught by the calling application and retried, but the driver object is still usable
    /// after the error.
    ///
    /// I've observed occasional desyncs where the read of the full packet times out, but remaining
    /// bytes from that packet are returned on the next read. This may be due to shoddy counterfeit
    /// USB-Serial cables.
    fn wait_for_start(&mut self) -> Result<u8> {
        let mut start = [0u8; 1];
        loop {
            std::io::Read::read_exact(&mut self.port, &mut start)?;
            if start[0] == START_BYTE {
                return Ok(start[0]);
            }
        }
    }

    /// Receive a response from the reader
    fn receive(&mut self) -> Result<Response> {
        let start = self.wait_for_start()?;
        let mut len = [0u8; 1];
        std::io::Read::read_exact(&mut self.port, &mut len)?;
        let len = len[0] as usize;
        let mut response: Vec<u8> = Vec::with_capacity(len + 2);
        response.extend(&[start, len as u8]);
        {
            let reference = self.port.by_ref();
            reference.take(len as u64).read_to_end(&mut response)?;
        }
        debug!("Receive: {:?}", response);
        Ok(Response::from_bytes(response)?)
    }

    fn exchange(&mut self, command: Command) -> Result<Response> {
        self.send(command)?;
        self.receive()
    }

    /// Send a command with no parameters and receive a response
    fn exchange_simple(&mut self, command: CommandType) -> Result<Response> {
        let cmd = Command {
            address: self.address,
            command: command,
            data: vec![],
        };
        self.exchange(cmd)
    }

    /// Reset the reader
    pub fn reset(&mut self) -> Result<()> {
        self.exchange_simple(CommandType::Reset)?;
        Ok(())
    }

    /// Get the firmware version of the reader
    ///
    /// Returns a tuple of (major, minor).
    pub fn get_version(&mut self) -> Result<(u8, u8)> {
        let response = self.exchange_simple(CommandType::GetFirmwareVersion)?;
        Ok((response.data[0], response.data[1]))
    }

    /// Set the working antenna ID
    ///
    /// `antenna_id` is from 0 to the number of available antennas.
    pub fn set_work_antenna(&mut self, antenna_id: u8) -> Result<()> {
        let cmd = Command {
            address: self.address,
            command: CommandType::SetWorkAntenna,
            data: vec![antenna_id],
        };
        self.exchange(cmd)?;
        Ok(())
    }

    /// Get the working antenna ID
    ///
    /// Returns an ID from 0 to the number of available antennas.
    pub fn get_work_antenna(&mut self) -> Result<u8> {
        let response = self.exchange_simple(CommandType::GetWorkAntenna)?;
        Ok(response.data[0])
    }

    /// Get the state of the antenna connection detector for the working antenna
    ///
    /// The value is the detector threshold in dB, or 0 if disabled.
    pub fn get_antenna_connection_detector(&mut self) -> Result<i8> {
        let response = self.exchange_simple(CommandType::GetAntConnectionDetector)?;
	Ok(-(response.data[0] as i8))
    }

    /// Set the output power per antenna and save to flash
    ///
    /// The length of `power` should be the number of antennas, and the value of power
    /// is in dBm (acceptable range is reader-dependent).
    pub fn set_output_power(&mut self, power: &[u8]) -> Result<()> {
        assert_eq!(power.len(), self.antenna_count);
        let cmd = Command {
            address: self.address,
            command: CommandType::SetOutputPower,
            data: power.to_vec(),
        };
        self.exchange(cmd)?;
        Ok(())
    }

    /// Get the output power per antenna
    ///
    /// Returns a vector of power for each antenna (in dBm)
    pub fn get_output_power(&mut self) -> Result<Vec<u8>> {
        let response = self.exchange_simple(CommandType::GetOutputPower)?;
        if response.data.len() == 1 {
            // Reader only sends the power once if all antennas are set the same,
            // so repeat it for consistency.
            return Ok(iter::repeat(response.data[0])
                .take(self.antenna_count)
                .collect());
        }
        Ok(response.data)
    }

    /// Fetch the temperature of the reader in celsius
    pub fn get_temperature(&mut self) -> Result<i8> {
        let response = self.exchange_simple(CommandType::GetReaderTemperature)?;
        let mut temp = response.data[1] as i8;
        // Datasheet says the first byte is 0x01 if negative, but this doesn't
        // seem to be correct. Guessing they got that reversed. It's not that cold in here.
        if response.data[0] == 0x00 {
            temp = -temp;
        }
        Ok(temp)
    }

    /// Measure the return loss in dB of the selected antenna
    pub fn measure_return_loss(&mut self, frequency: f32) -> Result<i8> {
        let cmd = Command {
            address: self.address,
            command: CommandType::GetRFPortReturnLoss,
            data: vec![convert_from_frequency(frequency)?],
        };
        let response = self.exchange(cmd)?;
        Ok(-(response.data[0] as i8))
    }

    /// Start an inventory operation on the selected antenna and return inventory data in real time.
    ///
    /// The `repeat` parameter appears to indicate the number of attempts the reader will make
    /// (although this is unclear - the datasheet calls this "repeat time"). It can be set to 255
    /// which means the reader will optimise this for speed to allow fast multi-antenna operation.
    pub fn real_time_inventory(&mut self, repeat: u8) -> Result<InventoryResult> {
        let cmd = Command {
            address: self.address,
            command: CommandType::RealTimeInventory,
            data: vec![repeat],
        };
        self.send(cmd)?;

        let mut tags: Vec<InventoryItem> = Vec::new();
        loop {
            let response = self.receive()?;
            if response.data.len() < 8 {
                return InventoryResult::from_bytes(&response.data, tags);
            };
            tags.push(InventoryItem::from_bytes(&response.data)?);
        }
    }
}
