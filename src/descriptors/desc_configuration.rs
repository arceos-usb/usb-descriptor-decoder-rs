use alloc::vec::Vec;

use super::desc_interface::TopologyUSBFunction;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Configuration {
    length: u8,
    ty: u8,
    total_length: u16,
    num_interfaces: u8,
    config_val: u8,
    config_string: u8,
    attributes: u8,
    max_power: u8,
}
impl Configuration {
    pub fn config_val(&self) -> u8 {
        self.config_val
    }
    pub fn length(&self) -> u8 {
        self.length
    }
    pub fn ty(&self) -> u8 {
        self.ty
    }
    pub fn total_length(&self) -> u16 {
        self.total_length
    }
    pub fn num_interfaces(&self) -> u8 {
        self.num_interfaces
    }
    pub fn config_string(&self) -> u8 {
        self.config_string
    }
    pub fn attributes(&self) -> u8 {
        self.attributes
    }
    pub fn max_power(&self) -> u8 {
        self.max_power
    }
}

#[derive(Debug)]
pub struct TopologyConfigDesc {
    pub desc: Configuration,
    pub functions: Vec<TopologyUSBFunction>,
}
