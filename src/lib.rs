extern crate num_enum;
extern crate serial;

pub mod error;
mod protocol;

use serial::core::prelude::*;
use std::convert::TryFrom;
use std::time::Duration;

use crate::protocol::{CommandType, ResponseCode};
use crate::error::{Error, Result};

pub struct Reader {
    port: serial::SystemPort,
    address: u8,
}

#[derive(PartialEq, Debug)]
struct Command {
    address: u8,
    command: CommandType,
    data: Vec<u8>,
}

impl Command {
    fn to_bytes(&self) -> Vec<u8> {
        let pkt_len = self.data.len() + 3;
        let mut pkt: Vec<u8> = Vec::new();
        pkt.push(0xA0); // Start byte
        pkt.push(pkt_len as u8); // Length byte
        pkt.push(self.address);
        pkt.push(self.command as u8);
        pkt.append(&mut self.data.clone());
        pkt.push(Reader::calculate_checksum(&pkt));
        pkt
    }
}

#[derive(PartialEq, Debug)]
struct Response {
    address: u8,
    status: ResponseCode,
    data: Vec<u8>,
}

impl Response {
    fn from_bytes(bytes: &[u8]) -> Result<Response> {
        assert_eq!(bytes[0] as usize, bytes.len() - 1);
        let len = bytes.len();

        /*
        let crc = Reader::calculate_crc(&bytes[0..len - 2]);
        let payload_crc: u16 = ((bytes[len - 1] as u16) << 8) + bytes[len - 2] as u16;
        if payload_crc != crc {
            return Err(Error::Program("Bad CRC".to_string()));
        }
        */

        let payload = &bytes[1..len - 1];
        Ok(Response {
            address: payload[0],
            status: ResponseCode::try_from(payload[2]).unwrap(),
            data: payload[2..].to_vec(),
        })
    }
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
            address: 0,
        })
    }

    fn calculate_checksum(data: &[u8]) -> u8 {
        let mut sum: u8 = 0;

        for i in 0..data.len() {
            let (newsum, _) = sum.overflowing_add(data[i]);
            sum = newsum;
        }
        !(sum) + 1
    }

    fn send_receive(&mut self, cmd: Command) -> Result<Response> {
        let cmd_bytes = cmd.to_bytes();
        println!("Send {:?}", cmd_bytes);
        std::io::Write::write(&mut self.port, &cmd_bytes)?;
        let mut header = [0u8; 2];
        std::io::Read::read_exact(&mut self.port, &mut header)?;
        if header[0] != 0xA0 {
            return Err(Error::Program(format!("Invalid header byte {:?}", header[0])));
        }
        let len = header[1];
        let mut response: Vec<u8> = Vec::with_capacity(len as usize + 1);
        response.push(len);
        {
            use std::io::Read;
            let reference = self.port.by_ref();
            reference.take(len as u64).read_to_end(&mut response)?;
        }
        let response = Response::from_bytes(&response)?;
        if !response.status.is_success() {
            return Err(Error::from(response.status));
        }
        Ok(response)
    }

    pub fn reset(&mut self) -> Result<()> {
        let cmd = Command {
            address: self.address,
            command: CommandType::Reset,
            data: vec![]
        };
        self.send_receive(cmd)?;
        Ok(())
    }

    pub fn get_version(&mut self) -> Result<(u8, u8)> {
        let cmd = Command {
            address: self.address,
            command: CommandType::GetFirmwareVersion,
            data: vec![]
        };
        let response = self.send_receive(cmd)?;
        Ok((response.data[0], response.data[1]))
    }
}

#[test]
fn test_checksum() {
    // Test vectors generated using example C code from datasheet
    assert_eq!(Reader::calculate_checksum(&[1, 2, 3, 4]), 246);
    assert_eq!(Reader::calculate_checksum(&[134, 200, 3, 253]), 178);
    assert_eq!(Reader::calculate_checksum(&[220, 4, 3, 30]), 255);
    assert_eq!(Reader::calculate_checksum(&[20, 45, 3, 30, 150, 230, 120]), 170);
}
