use num_derive::FromPrimitive;

use crate::TopologyDescriptor;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Hid {
    pub len: u8,
    pub descriptor_type: u8,
    pub hid_bcd: u16,
    pub country_code: u8,
    pub num_descriptions: u8,
    pub report_descriptor_type: u8, // actually, these two entry is a variant length vector, but we only pick first entry!
    pub report_descriptor_len: u16, //
}

impl TopologyDescriptor for Hid {
    fn desc_type(&self) -> u8 {
        self.descriptor_type
    }

    fn actual_len(&self) -> crate::Offset {
        self.len as _
    }
}

#[derive(FromPrimitive, Copy, Clone, Debug)]
#[repr(u8)]
pub enum USBHIDSubclassDescriptorType {
    None = 0,
    BootInterface = 1,
}

#[derive(FromPrimitive, Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum USBHIDProtocolDescriptorType {
    None = 0,
    KeyBoard = 1,
    Mouse = 2,
}

#[derive(FromPrimitive, Copy, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum HIDDescriptorTypes {
    //HID
    Hid = 0x21,
    // HIDReport = 0x22,
    // HIDPhysical = 0x23,
}
#[derive(Debug)]
pub struct ReportEvent {
    pub usage_page: u32,
    pub usage: u32,
    pub value: i32,
    pub relative: bool,
}

// #[derive(Debug)]
// pub struct ReportInput {
//     pub bit_length: usize,
//     pub bit_offset: usize,
//     pub global_state: GlobalItemsState,
//     pub local_state: LocalItemsState,
//     pub flags: MainItemFlags,
// }

// pub struct ReportHandler {
//     pub inputs: Vec<ReportInput>,
//     pub total_byte_length: usize,
//     pub absolutes: HashMap<(u32, u32), i32>,
//     pub arrays: HashSet<(u32, u32)>,
// }

// impl ReportHandler {
//     pub fn new() -> Result<self, Error> {}
// }
