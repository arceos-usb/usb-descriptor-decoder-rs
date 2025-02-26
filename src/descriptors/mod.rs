use alloc::vec::Vec;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use crate::ParserError;

pub mod desc_configuration;
pub mod desc_device;
pub mod desc_endpoint;
pub mod desc_interface;
pub mod desc_str;
pub mod parser;

#[cfg(feature = "hid")]
pub mod desc_hid;
#[cfg(feature = "uvc")]
pub mod desc_uvc;

#[allow(non_camel_case_types)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum USBStandardDescriptorTypes {
    //USB 1.1: 9.4 Standard Device Requests, Table 9-5. Descriptor Types
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    // USB 2.0: 9.4 Standard Device Requests, Table 9-5. Descriptor Types
    DeviceQualifier = 0x06,
    OtherSpeedConfiguration = 0x07,
    InterfacePower1 = 0x08,
    // USB 3.0+: 9.4 Standard Device Requests, Table 9-5. Descriptor Types
    OTG = 0x09,
    Debug = 0x0a,
    InterfaceAssociation = 0x0b,
    Bos = 0x0f,
    DeviceCapability = 0x10,
    SuperSpeedEndpointCompanion = 0x30,
    SuperSpeedPlusIsochEndpointCompanion = 0x31,
}

impl USBStandardDescriptorTypes {
    ///always peek first data chunk
    pub fn peek_type(data: &[u8]) -> Result<(USBStandardDescriptorTypes, u8), ParserError> {
        USBStandardDescriptorTypes::from_u8(data[1])
            .ok_or_else(|| {
                if data[0] == 0 && data[1] == 0 {
                    ParserError::Ended
                } else {
                    ParserError::PeekFailed(data[1].clone())
                }
            })
            .map(|desc_type| (desc_type, data[0]))
    }
}

#[derive(Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum PortSpeed {
    FullSpeed = 1,
    LowSpeed = 2,
    HighSpeed = 3,
    SuperSpeed = 4,
    SuperSpeedPlus = 5,
}
