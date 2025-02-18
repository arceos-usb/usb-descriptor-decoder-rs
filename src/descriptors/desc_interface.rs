use alloc::{boxed::Box, string::String, vec::Vec};

use crate::TopologyDescriptor;

use super::desc_endpoint::Endpoint;

#[derive(Copy, Clone, Default, Debug)]
#[repr(C, packed)]
pub struct Interface {
    pub len: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface: u8,
}
impl Interface {
    pub fn ty(&self) -> (u8, u8, u8) {
        (
            self.interface_class,
            self.interface_subclass,
            self.interface_protocol,
        )
    }
}

#[derive(Copy, Clone, Default, Debug)]
#[repr(C, packed)]
pub struct InterfaceAssociation {
    pub len: u8,
    pub descriptor_type: u8,
    pub first_interface: u8,
    pub interface_count: u8,
    pub function_class: u8,
    pub function_subclass: u8,
    pub function_protocol: u8,
    pub function: u8,
}

pub struct USBInterface {
    interface: Interface,
    endpoints: Vec<Endpoint>,
    flag: String,
}

pub struct ExtraDesc(Vec<Box<dyn TopologyDescriptor>>);

pub enum TopologyUSBFunction {
    Interface(Vec<USBInterface>, ExtraDesc),
    InterfaceAssociation(InterfaceAssociation, Vec<USBInterface>, ExtraDesc),
}
