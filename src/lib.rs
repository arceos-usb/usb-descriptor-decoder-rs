#![no_std]
#![feature(if_let_guard, let_chains)]
#![allow(dead_code)]

use core::fmt::Debug;

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
        let descriptor_decoder = DescriptorDecoder {
            modules: BTreeMap::new(),
        };
        contained_parsers::load_embed_parsers(descriptor_decoder)
    }

    pub fn add_module(&mut self, module: Box<dyn DescriptorDecoderModule>) {
        let str = module.function_name_string();
        self.modules.insert(str, module);
    }
}

pub type Offset = usize;

pub trait DescriptorDecoderModule: Send + Sync {
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
    NotEnoughEndpoints,
    NoSuitableModule,
    PeekFailed(u8),
    NotSupportedDescriptorCombination(String),
    Ended,
}

pub trait TopologyDescriptor: Debug + Send + Sync {
    fn desc_type(&self) -> u8;
    fn subtype(&self) -> Option<u8> {
        return None;
    }
    fn actual_len(&self) -> Offset;
}
