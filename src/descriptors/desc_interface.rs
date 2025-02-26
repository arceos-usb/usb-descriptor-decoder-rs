use alloc::{boxed::Box, string::String, vec::Vec};

use crate::TopologyDescriptor;

use super::desc_endpoint::Endpoint;

pub type InterfaceIdentifier = (u8, u8, u8);
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
    pub fn ty(&self) -> InterfaceIdentifier {
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
impl InterfaceAssociation {
    pub fn ty(&self) -> InterfaceIdentifier {
        (
            self.function_class,
            self.function_subclass,
            self.function_protocol,
        )
    }
}

#[derive(Debug)]
pub struct USBInterface {
    pub interface: Interface,
    pub endpoints: Vec<Endpoint>,
    pub flag: String,
    pub extra: ExtraDesc,
}

pub type ExtraDesc = Vec<Box<dyn TopologyDescriptor>>;

#[derive(Debug)]
pub enum TopologyUSBFunction {
    Interface(Vec<USBInterface>),
    InterfaceAssociation(InterfaceAssociation, Vec<Vec<USBInterface>>, ExtraDesc),
}

impl USBInterface {
    pub fn is_alternative(&self, ident: &Interface) -> bool {
        self.interface.interface_number == ident.interface_number
    }
}
