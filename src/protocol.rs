use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use bitreader::BitReader;

use crate::error::{Error, Result};

pub(crate) const START_BYTE: u8 = 0xA0;

#[derive(Copy, Clone, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
pub(crate) enum CommandType {
    // Reader commands
    Reset = 0x70,
    SetUARTBaudRate = 0x71,
    GetFirmwareVersion = 0x72,
    SetReaderAddress = 0x73,
    SetWorkAntenna = 0x74,
    GetWorkAntenna = 0x75,
    SetOutputPower = 0x76,
    GetOutputPower = 0x77,
    SetFrequencyRegion = 0x78,
    GetFrequencyRegion = 0x79,
    SetBeeperMode = 0x7A,
    GetReaderTemperature = 0x7B,
    ReadGPIOValue = 0x60,
    WriteGPIOValue = 0x61,
    SetAntConnectionDetector = 0x62,
    GetAntConnectionDetector = 0x63,
    SetTemporaryOutputPower = 0x66,
    SetReaderIdentifier = 0x67,
    GetReaderIdentifier = 0x68,
    SetRFLinkProfile = 0x69,
    GetRFLinkProfile = 0x6A,
    GetRFPortReturnLoss = 0x7E,

    // ISO18000-6C Commands
    Inventory = 0x80,
    Read = 0x81,
    Write = 0x82,
    Lock = 0x83,
    Kill = 0x84,
    SetAccessEPCMatch = 0x85,
    GetAccessEPCMatch = 0x86,
    RealTimeInventory = 0x89,
    FastSwitchAntInventory = 0x8A,
    CustomizedSessionTargetInventory = 0x8B,
    SetImpinjFastTID = 0x8C,
    SetAndSaveImpinjFastTIC = 0x8D,
    GetImpinjFastTID = 0x8E,

    // ISO18000-6B Commands
    Inventory6B = 0xB0,
    Read6B = 0xB1,
    Write6B = 0xB2,
    Lock6B = 0xB3,
    QueryLock6B = 0xB4,

    // Buffer Control Commands
    GetInventoryBuffer = 0x90,
    GetAndResetInventoryBuffer = 0x91,
    GetBufferTagCount = 0x92,
    ResetInventoryBuffer = 0x93,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum ResponseCode {
    Success = 0x00,
    Fail = 0x11,

    MCUResetError = 0x20,
    CWOnError = 0x21,
    AntennaMissingError = 0x22,
    WriteFlashError = 0x23,
    ReadFlashError = 0x24,
    SetOutputPowerError = 0x25,

    TagInventoryError = 0x31,
    TagReadError = 0x32,
    TagWriteError = 0x33,
    TagLockError = 0x34,
    TagKillError = 0x35,
    NoTagError = 0x36,
    InventoryOKAccessFailError = 0x37,
    BufferEmptyError = 0x38,

    AccessFailError = 0x40,
    InvalidParameterError = 0x41,
    WordCntTooLongError = 0x42,
    MemBankOutOfRangeError = 0x43,
    LockRegionOutOfRangeError = 0x44,
    LockTypeOutOfRangeError = 0x45,
    InvalidReaderAddressError = 0x46,
    InvalidAntennaIDError = 0x47,
    OutputPowerOutOfRangeError = 0x48,
    InvalidFrequencyRegionError = 0x49,
    InvalidBaudRateError = 0x4A,
    InvalidBeeperModeError = 0x4B,
    EPCMatchLenTooLongError = 0x4C,
    EPCMatchLenError = 0x4D,
    InvalidEPCMatchModeError = 0x4E,
    InvalidFrequencyRangeError = 0x4F,
    FailToGetRN16Error = 0x50,
    InvalidDRMModeError = 0x51,
    PLLLockFailError = 0x52,
    RFChipError = 0x53,
    FailToAchieveDesiredPowerError = 0x54,
    CopyrightAuthenticationError = 0x55,
    SpectrumRegulationError = 0x56,
    OutputPowerTooLowError = 0x57,
}

/// Whether this command includes a response type in its reply.
///
/// Hilariously in some cases this depends on the length of the response packet.
fn command_has_response_code(command: CommandType, length: usize) -> bool {
    match command {
        CommandType::GetFirmwareVersion => false,
        CommandType::GetOutputPower => false,
        CommandType::GetReaderTemperature => false,
        CommandType::RealTimeInventory => {
            if length == 0x04 {
                // Failed inventory
                true
            } else {
                false
            }
        }
        _ => true   
    }
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum FrequencyRegion {
    FCC = 0x01,
    ETSI = 0x02,
    CHN = 0x03,
    UserDefined = 0x04,
}


/// Convert internal representation to a frequency in MHz
///
/// This is derived from table 4 in the datasheet.
fn convert_frequency(freq: u8) -> f32 {
    if freq < 7 {
        return 865. + 0.5 * freq as f32;
    }
    return 902. + 0.5 * freq as f32;
}

/// Convert internal representation to a RSSI in dBm
///
/// This is derived from table 5 in the datasheet.
fn convert_rssi(rssi: u8) -> i8 {
    // Is this discontinuity a bug? Who knows.
    if rssi > 89 {
        (rssi as i16 - 129) as i8
    } else {
        (rssi as i16 - 130) as i8
    }
}

/// Calculate checksum digit
///
/// Datasheet section 6
fn calculate_checksum(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;

    for i in 0..data.len() {
        let (newsum, _) = sum.overflowing_add(data[i]);
        sum = newsum;
    }
    let (result, _) = (!sum).overflowing_add(1);
    result
}

#[derive(PartialEq, Debug)]
pub(crate) struct Command {
    pub address: u8,
    pub command: CommandType,
    pub data: Vec<u8>,
}

impl Command {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        // Packet length excluding start and length bytes
        let pkt_len: usize = self.data.len() + 3;
        let mut pkt: Vec<u8> = Vec::with_capacity(pkt_len + 2);
        pkt.push(START_BYTE);
        pkt.push(pkt_len as u8); // Length byte
        pkt.push(self.address);
        pkt.push(self.command as u8);
        pkt.append(&mut self.data.clone());
        pkt.push(calculate_checksum(&pkt));
        pkt
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct Response {
    pub address: u8,
    pub command: u8,
    pub status: Option<ResponseCode>,
    pub data: Vec<u8>,
}

impl Response {
    pub(crate) fn from_bytes(data: Vec<u8>) -> Result<Response> {
        assert_eq!(data[0], START_BYTE);
        assert_eq!(data[1] as usize, data.len() - 2);
        let len = data.len();

        let checksum = calculate_checksum(&data[0..len - 1]);
        if data[len - 1] != checksum {
            return Err(Error::Program(format!(
                "Bad checksum: got {:?}, expecting {:?}",
                data[len], checksum
            )));
        }
        let command_type = CommandType::try_from(data[3])?;
        
        // Some responses have a response code, some don't.
        let mut data_offset = 4;
        let mut response_code = None;

        if command_has_response_code(command_type, len) {
            data_offset = 5;
            response_code = Some(ResponseCode::try_from(data[4])?);
        }

        Ok(Response {
            address: data[2],
            command: data[3],
            status: response_code,
            data: data[data_offset..len - 1].to_owned(),
        })
    }

    pub(crate) fn is_success(&self) -> bool {
        match self.status {
            Some(ResponseCode::Success) => true,
            None => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct InventoryItem {
    pub frequency: f32,
    pub antenna: u8,
    pub pc: Vec<u8>,
    pub epc: Vec<u8>,
    pub rssi: i8
}

impl InventoryItem {
    pub(crate) fn from_bytes(data: &[u8]) -> Result<InventoryItem> {
        let first_byte = [data[0]];
        let mut reader = BitReader::new(&first_byte);
        let len = data.len();
        Ok(InventoryItem{
            frequency: convert_frequency(reader.read_u8(6)?),
            antenna: reader.read_u8(2)?,
            pc: data[1..3].to_owned(),
            epc: data[3..len-2].to_owned(),
            rssi: convert_rssi(data[len-1])
        })
    }
}


#[derive(PartialEq, Debug)]
pub struct InventoryResult {
    pub items: Vec<InventoryItem>,
    pub antenna: u8,
    pub read_rate: u16,
    pub total_read: u32,
}

impl InventoryResult {
    pub(crate) fn from_bytes(data: &[u8], items: Vec<InventoryItem>) -> Result<InventoryResult> {
        let mut reader = BitReader::new(data);
        Ok(InventoryResult{
            items: items,
            antenna: reader.read_u8(8)?,
            read_rate: reader.read_u16(16)?,
            total_read: reader.read_u32(32)?
        })
    }
}

#[test]
fn test_checksum() {
    // Test vectors generated using example C code from datasheet
    assert_eq!(calculate_checksum(&[1, 2, 3, 4]), 246);
    assert_eq!(calculate_checksum(&[134, 200, 3, 253]), 178);
    assert_eq!(calculate_checksum(&[220, 4, 3, 30]), 255);
    assert_eq!(calculate_checksum(&[20, 45, 3, 30, 150, 230, 120]), 170);
    assert_eq!(calculate_checksum(&[0xA0, 0x03, 0x01, 0x72]), 0xEA);
}
