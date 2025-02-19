use core::{ops::AddAssign, ptr, usize};

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::String, vec::Vec};
use log::error;
use num_traits::Zero;

use crate::{DescriptorDecoder, Offset, ParserError, TopologyDescriptor};

use super::{
    desc_configuration::{Configuration, TopologyConfigDesc},
    desc_device::{Device, StandardUSBDeviceClassCode, TopologyDeviceDesc},
    desc_endpoint::Endpoint,
    desc_interface::{
        Interface, InterfaceAssociation, InterfaceIdentifier, TopologyUSBFunction, USBInterface,
    },
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

    pub fn peek_device_desc(input: Vec<u8>) -> Result<Device, ParserError> {
        match USBStandardDescriptorTypes::peek_type(&input[..2]) {
            Ok((desc_type, len)) if desc_type == USBStandardDescriptorTypes::Device => {
                let device: Device = Self::cast(&input[0..len as _]);
                Ok(device)
            }
            _ => Err(ParserError::NotDeviceDescriptor),
        }
    }

    pub fn cast<T>(input: &[u8]) -> T {
        let raw: *const [u8] = input;
        unsafe { ptr::read(raw as *const T) }
    }

    pub fn parse_config(&self, input: &[u8]) -> Result<(TopologyConfigDesc, Offset), ParserError> {
        match USBStandardDescriptorTypes::peek_type(&input[..2]) {
            Ok((desc_type, len)) if desc_type == USBStandardDescriptorTypes::Configuration => {
                let config: Configuration = Self::cast(&input[0..len as _]);
                let mut functions: Vec<TopologyUSBFunction> = Vec::new();
                let mut offset: usize = len as _;

                loop {
                    match USBStandardDescriptorTypes::peek_type(&input[offset..]) {
                        Ok((USBStandardDescriptorTypes::Interface, len_inner)) => {
                            let (function, l) =
                                self.try_parse_function(&input[offset..], len_inner as _)?;
                            functions.push(function);
                            offset += l;
                        }
                        Ok((USBStandardDescriptorTypes::InterfaceAssociation, len_inner)) => {
                            let (function, l) = self.try_parse_function_interface_association(
                                &input[offset..],
                                len_inner as _,
                            )?;
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
            any => {
                error!("{:#?}", any);
                Err(ParserError::NotConfigDescriptor)
            }
        }
    }

    fn try_parse_function(
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

    fn try_parse_function_interface_association(
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

pub fn parse_endpoint(input: &[u8]) -> Result<(Endpoint, Offset), ParserError> {
    if let Ok((USBStandardDescriptorTypes::Endpoint, len)) =
        USBStandardDescriptorTypes::peek_type(input)
    {
        let cast: Endpoint = DescriptorDecoder::cast(&input[..len as _]);
        Ok((cast, len as _))
    } else {
        Err(ParserError::NotEndpoint)
    }
}

pub fn parse_single_interface(input: &[u8]) -> Result<Interface, ParserError> {
    if let Ok((USBStandardDescriptorTypes::Interface, len)) =
        USBStandardDescriptorTypes::peek_type(input)
    {
        let cast: Interface = DescriptorDecoder::cast(&input[..len as _]);
        Ok(cast)
    } else {
        Err(ParserError::NotFunction)
    }
}

pub fn parse_interface_association(input: &[u8]) -> Result<InterfaceAssociation, ParserError> {
    if let Ok((USBStandardDescriptorTypes::InterfaceAssociation, len)) =
        USBStandardDescriptorTypes::peek_type(input)
    {
        let cast: InterfaceAssociation = DescriptorDecoder::cast(&input[..len as _]);
        Ok(cast)
    } else {
        Err(ParserError::NotFunction)
    }
}

///should gurantee checked that has interface at head, and not IA
pub unsafe fn parse_interface_group<P>(
    input: &[u8],
    extra_parser: P,
    offset: &mut usize,
    flag: String,
) -> Result<Vec<USBInterface>, ParserError>
where
    P: Fn(&[u8]) -> Option<Box<dyn TopologyDescriptor>>,
{
    let mut result = Vec::new();

    let required_interface_number = {
        let interface: Interface = DescriptorDecoder::cast(&input[..*offset as _]);
        let mut remain_ep = interface.num_endpoints;
        let mut endpoints = Vec::new();
        let mut extra = Vec::new();
        loop {
            match USBStandardDescriptorTypes::peek_type(&input[*offset..]) {
                Ok((USBStandardDescriptorTypes::Endpoint, _)) => {
                    let (ep, len) = parse_endpoint(&input[*offset..])?;
                    offset.add_assign(len);
                    remain_ep -= 1;
                    endpoints.push(ep);
                }
                Err(ParserError::PeekFailed(_))
                    if let Some(parsed) = extra_parser(&input[*offset..]) =>
                {
                    offset.add_assign(parsed.actual_len());
                    extra.push(parsed);
                }
                _ => break,
            }
        }

        if remain_ep != 0 {
            error!("not enough endpoints: {}", remain_ep);
            return Err(ParserError::NotEnoughEndpoints);
        }

        let interface_number = interface.interface_number;
        result.push(USBInterface {
            interface,
            endpoints,
            flag: flag.clone(),
            extra,
        });
        interface_number
    };

    loop {
        if let Ok((USBStandardDescriptorTypes::Interface, len)) =
            USBStandardDescriptorTypes::peek_type(&input[*offset..])
        {
            let interface: Interface = DescriptorDecoder::cast(&input[..*offset as _]);
            if interface.interface_number == required_interface_number {
                offset.add_assign(len as usize);

                let mut remain_ep = interface.num_endpoints;
                let mut endpoints = Vec::new();
                let mut extra = Vec::new();
                loop {
                    match USBStandardDescriptorTypes::peek_type(&input[*offset..]) {
                        Ok((USBStandardDescriptorTypes::Endpoint, _)) => {
                            let (ep, len) = parse_endpoint(&input[*offset..])?;
                            offset.add_assign(len);
                            remain_ep -= 1;
                            endpoints.push(ep);
                        }
                        Err(ParserError::PeekFailed(_))
                            if let Some(parsed) = extra_parser(&input[*offset..]) =>
                        {
                            offset.add_assign(parsed.actual_len());
                            extra.push(parsed);
                        }
                        _ => break,
                    }
                }

                if remain_ep != 0 {
                    error!("not enough endpoints: {}", remain_ep);
                    return Err(ParserError::NotEnoughEndpoints);
                }

                result.push(USBInterface {
                    interface,
                    endpoints,
                    flag: flag.clone(),
                    extra,
                });
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use alloc::{format, vec};
    use log::{debug, error, info};

    use crate::{descriptors::desc_device::Device, DescriptorDecoder};

    #[test]
    fn test_parse_config() {
        let input = [
            9u8, 2, 34, 0, 1, 1, 6, 160, 50, 9, 4, 0, 0, 1, 3, 1, 2, 0, 9, 33, 1, 0, 0, 1, 34, 52,
            0, 7, 5, 129, 3, 4, 0, 7, 0, 0,
        ];

        let descriptor_decoder = DescriptorDecoder::new();
        let (result, offset) = descriptor_decoder.parse_config(&input[..]).unwrap();
        let formatted = format!("{:?}", result);

        assert_eq!(formatted, "TopologyConfigDesc { desc: Configuration { length: 9, ty: 2, total_length: 34, num_interfaces: 1, config_val: 1, config_string: 6, attributes: 160, max_power: 50 }, functions: [Interface([USBInterface { interface: Interface { len: 9, descriptor_type: 4, interface_number: 0, alternate_setting: 0, num_endpoints: 1, interface_class: 3, interface_subclass: 1, interface_protocol: 2, interface: 0 }, endpoints: [Endpoint { len: 7, descriptor_type: 5, endpoint_address: 129, attributes: 3, max_packet_size: 4, interval: 7, ssc: None }], flag: \"hid\", extra: [Hid { len: 9, descriptor_type: 33, hid_bcd: 1, country_code: 0, num_descriptions: 1, report_descriptor_type: 34, report_descriptor_len: 52 }] }])] }")
        // info!("{formatted}")
    }

    #[test]
    fn test_peek_device() {
        let input = vec![18, 1, 0, 2, 0, 0, 0, 64, 39, 6, 1, 0, 0, 0, 1, 2, 9, 1];

        let device = DescriptorDecoder::peek_device_desc(input).unwrap();

        assert_eq!(
            device,
            Device {
                len: 18,
                descriptor_type: 1,
                cd_usb: 512,
                class: 0,
                subclass: 0,
                protocol: 0,
                max_packet_size0: 64,
                vendor: 1575,
                product_id: 1,
                device: 0,
                manufacture: 1,
                product: 2,
                serial_number: 9,
                num_configurations: 1,
            }
        )
    }
}
