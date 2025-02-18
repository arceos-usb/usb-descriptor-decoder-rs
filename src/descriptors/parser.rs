use core::{ptr, usize};

use alloc::vec::Vec;

use crate::{DescriptorDecoder, Offset, ParserError, TopologyDescriptor};

use super::{
    desc_configuration::{Configuration, TopologyConfigDesc},
    desc_device::{Device, TopologyDeviceDesc},
    desc_interface::{Interface, InterfaceAssociation, TopologyUSBFunction, USBInterface},
    USBStandardDescriptorTypes,
};

impl DescriptorDecoder {
    pub fn parse(&self, input: Vec<u8>) -> Result<TopologyDeviceDesc, ParserError> {
        match USBStandardDescriptorTypes::peek_type(&input[..2]) {
            Ok((desc_type, len)) if desc_type == USBStandardDescriptorTypes::Device => {
                let device: Device = Self::cast(&input[0..len as _]);
                let mut configs: Vec<TopologyConfigDesc> = Vec::new();

                let mut offset: usize = len as _;
                for _ in 0..device.num_configurations {
                    let (config, l) = self.parse_config(&input[offset..])?;
                    configs.push(config);
                    offset += l;
                }

                Ok(TopologyDeviceDesc {
                    desc: device,
                    configs,
                })
            }
            _ => Err(ParserError::NotDeviceDescriptor),
        }
    }

    fn cast<T>(input: &[u8]) -> T {
        let raw: *const [u8] = input;
        unsafe { ptr::read(raw as *const T) }
    }

    fn parse_config(&self, input: &[u8]) -> Result<(TopologyConfigDesc, Offset), ParserError> {
        match USBStandardDescriptorTypes::peek_type(&input[..2]) {
            Ok((desc_type, len)) if desc_type == USBStandardDescriptorTypes::Configuration => {
                let config: Configuration = Self::cast(&input[0..len as _]);
                let mut functions: Vec<TopologyUSBFunction> = Vec::new();
                let mut offset: usize = len as _;

                loop {
                    match USBStandardDescriptorTypes::peek_type(&input[offset..]) {
                        Ok((USBStandardDescriptorTypes::Interface, len_inner)) => {
                            let (function, l) =
                                self.parse_interface(&input[offset..], len_inner as _)?;
                            functions.push(function);
                            offset += l;
                        }
                        Ok((USBStandardDescriptorTypes::InterfaceAssociation, len_inner)) => {
                            let (function, l) =
                                self.parse_interface_association(&input[offset..], len_inner as _)?;
                            functions.push(function);
                            offset += l;
                        }
                        _ => {
                            if offset != len as usize {
                                break;
                            } else {
                                return Err(ParserError::NotFunction);
                            }
                        }
                    };
                }

                Ok((
                    TopologyConfigDesc {
                        desc: config,
                        functions,
                    },
                    offset,
                ))
            }
            _ => Err(ParserError::NotDeviceDescriptor),
        }
    }

    fn parse_interface(
        &self,
        input: &[u8],
        len: usize,
    ) -> Result<(TopologyUSBFunction, Offset), ParserError> {
        let cast: Interface = Self::cast(&input[..len]);
        let descriptor_decoder_module = self
            .modules
            .values()
            .filter(|module| {
                module.filter_triple(
                    cast.interface_class,
                    cast.interface_subclass,
                    cast.interface_protocol,
                )
            })
            .next()
            .ok_or(ParserError::NoSuitableModule)?;
        descriptor_decoder_module.parse(&input[..])
    }

    fn parse_interface_association(
        &self,
        input: &[u8],
        len: usize,
    ) -> Result<(TopologyUSBFunction, Offset), ParserError> {
        let cast: InterfaceAssociation = Self::cast(&input[..len]);
        let descriptor_decoder_module = self
            .modules
            .values()
            .filter(|module| {
                module.filter_triple(
                    cast.function_class,
                    cast.function_subclass,
                    cast.function_protocol,
                )
            })
            .next()
            .ok_or(ParserError::NoSuitableModule)?;
        descriptor_decoder_module.parse(&input[..])
    }
}
