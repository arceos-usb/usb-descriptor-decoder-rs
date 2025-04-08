use alloc::{boxed::Box, string::ToString};

use crate::{
    descriptors::{
        desc_device::StandardUSBDeviceClassCode,
        desc_hid::{HIDDescriptorTypes, Hid},
        desc_interface::TopologyUSBFunction,
        parser, USBStandardDescriptorTypes,
    },
    DescriptorDecoder, DescriptorDecoderModule, ParserError,
};

pub struct HIDParserModule {}

impl DescriptorDecoderModule for HIDParserModule {
    fn function_name_string(&self) -> alloc::string::String {
        "hid".to_string()
    }

    fn filter_triple(&self, class: u8, _subclass: u8, _protocol: u8) -> bool {
        class == StandardUSBDeviceClassCode::HID as u8
    }

    fn parse(
        &self,
        data: &[u8],
    ) -> Result<
        (
            crate::descriptors::desc_interface::TopologyUSBFunction,
            crate::Offset,
        ),
        crate::ParserError,
    > {
        let mut offset: usize = 0;
        match USBStandardDescriptorTypes::peek_type(data)? {
            (USBStandardDescriptorTypes::Interface, len) => {
                offset += len as usize;
                let usbinterfaces = unsafe {
                    parser::parse_interface_group(
                        &data[..],
                        |data| {
                            if data[1] == HIDDescriptorTypes::Hid as u8 {
                                let hid = DescriptorDecoder::cast::<Hid>(&data[..]);
                                return Some(Box::new(hid));
                            }
                            return None;
                        },
                        &mut offset,
                        "hid".to_string(),
                    )
                }?;

                return Ok((TopologyUSBFunction::Interface(usbinterfaces), offset));
            }
            (USBStandardDescriptorTypes::InterfaceAssociation, _) => {
                return Err(ParserError::NotSupportedDescriptorCombination(
                    "InterfaceAssociation in HID".to_string(),
                ))
            }
            _ => return Err(ParserError::NotFunction),
        }
    }
}
