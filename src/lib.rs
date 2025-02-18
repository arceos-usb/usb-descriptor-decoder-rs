#![no_std]
#![feature(if_let_guard, let_chains)]
#![allow(dead_code)]

use alloc::{boxed::Box, collections::btree_map::BTreeMap, string::String, vec::Vec};
use descriptors::{desc_device::TopologyDeviceDesc, desc_interface::TopologyUSBFunction};

extern crate alloc;

pub mod descriptors;

mod contained_parsers;

pub struct DescriptorDecoder {
    modules: BTreeMap<String, Box<dyn DescriptorDecoderModule>>,
}

impl DescriptorDecoder {
    pub fn new() -> DescriptorDecoder {
        DescriptorDecoder {
            modules: BTreeMap::new(),
        }
    }

    pub fn add_module(&mut self, module: Box<dyn DescriptorDecoderModule>) {
        let str = module.function_name_string();
        self.modules.insert(str, module);
    }
}

pub type Offset = usize;

pub trait DescriptorDecoderModule {
    fn function_name_string(&self) -> String;
    fn filter_triple(&self, class: u8, subclass: u8, protocol: u8) -> bool;
    ///return: TopologyUSBFunction,Len
    fn parse(&self, data: &[u8]) -> Result<(TopologyUSBFunction, Offset), ParserError>;
}

#[derive(Debug)]
pub enum ParserError {
    NotDeviceDescriptor,
    NotConfigDescriptor,
    NotFunction,
    NotEndpoint,
    NoSuitableModule,
    PeekFailed(u8),
}

pub trait TopologyDescriptor {
    fn desc_type(&self) -> u8;
}
