use num_derive::FromPrimitive;

#[derive(FromPrimitive, Copy, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum UVCVideoClassEndpointSubtypes {
    UNDEFINED = 0x00,
    GENERAL = 0x01,
    ENDPOINT = 0x02,
    INTERRUPT = 0x03,
}

#[derive(Clone, Debug)]
#[allow(non_camel_case_types)]
pub struct UVCVideoControlInterruptEndpoint {
    pub len: u8,
    pub descriptor_type: u8,
    pub descriptor_sub_type: UVCVideoClassEndpointSubtypes,
    pub max_transfer_size: u16,
}
