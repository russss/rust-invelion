extern crate bitreader;
extern crate log;
extern crate num_enum;
extern crate serial;

pub mod error;
mod protocol;

use log::debug;
use serial::core::prelude::*;
use std::io::Read;
use std::iter;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::protocol::{
    Command, CommandType, InventoryItem, InventoryResult, Response, START_BYTE,
};

pub struct Reader {
    port: serial::SystemPort,
    antenna_count: usize,
    address: u8,
}

impl Reader {
    pub fn new(port: &str) -> Result<Reader> {
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
            address: 1,
            antenna_count: 4,
        })
    }

    /// Send a command to the reader
    fn send(&mut self, cmd: Command) -> Result<()> {
        let cmd_bytes = cmd.to_bytes();
        debug!("Send: {:?}", cmd_bytes);
        std::io::Write::write(&mut self.port, &cmd_bytes)?;
        Ok(())
    }

    /// Receive a response from the reader
    fn receive(&mut self) -> Result<Response> {
        let mut header = [0u8; 2];
        std::io::Read::read_exact(&mut self.port, &mut header)?;
        if header[0] != START_BYTE {
            return Err(Error::Program(format!(
                "Invalid header byte {:?}",
                header[0]
            )));
        }
        let len = header[1] as usize;
        let mut response: Vec<u8> = Vec::with_capacity(len + 2);
        response.extend(&header);
        {
            let reference = self.port.by_ref();
            reference.take(len as u64).read_to_end(&mut response)?;
        }
        debug!("Receive: {:?}", response);
        let response = Response::from_bytes(response)?;
        if !response.is_success() {
            // is_success will only fail when response.status is present
            return Err(Error::from(response.status.unwrap()));
        }
        Ok(response)
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

    pub fn reset(&mut self) -> Result<()> {
        self.exchange_simple(CommandType::Reset)?;
        Ok(())
    }

    pub fn get_version(&mut self) -> Result<(u8, u8)> {
        let response = self.exchange_simple(CommandType::GetFirmwareVersion)?;
        Ok((response.data[0], response.data[1]))
    }

    pub fn set_work_antenna(&mut self, antenna_id: u8) -> Result<()> {
        let cmd = Command {
            address: self.address,
            command: CommandType::SetWorkAntenna,
            data: vec![antenna_id],
        };
        self.exchange(cmd)?;
        Ok(())
    }

    pub fn get_work_antenna(&mut self) -> Result<u8> {
        let response = self.exchange_simple(CommandType::GetWorkAntenna)?;
        Ok(response.data[0])
    }

    /// Set the output power per antenna and save to flash
    ///
    /// The length of `power` should be the number of antennas, and the value of power is 0-33 dBm.
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
    /// Returns a vector of power for each antenna (0-33 dBm)
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

    /// Fetch the temperature of the reader
    pub fn get_temperature(&mut self) -> Result<i8> {
        let response = self.exchange_simple(CommandType::GetReaderTemperature)?;
        let mut temp = response.data[1] as i8;
        // Datasheet says the first byte is 0x01 if negative, but this doesn't
        // seem to be correct. Guessing they got that reversed.
        if response.data[0] == 0x00 {
            temp = -temp;
        }
        Ok(temp)
    }

    pub fn inventory(&mut self, repeat: u8) -> Result<InventoryResult> {
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
