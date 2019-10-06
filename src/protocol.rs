use num_enum::TryFromPrimitive;

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(dead_code)]
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
    ResetInventoryBuffer = 0x93
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
    OutputPowerTooLowError = 0x57
}

impl ResponseCode {
    pub(crate) fn is_success(&self) -> bool {
        match self {
            ResponseCode::Success => true,
            _ => false,
        }
    }
}

