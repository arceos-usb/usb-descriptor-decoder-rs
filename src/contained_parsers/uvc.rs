use alloc::{boxed::Box, format, string::ToString, sync::Arc, vec::Vec};
use num_traits::FromPrimitive;

use crate::{
    descriptors::{
        desc_device::StandardUSBDeviceClassCode,
        desc_interface::{InterfaceAssociation, TopologyUSBFunction},
        desc_uvc::{
            uvc_endpoints::UVCVideoControlInterruptEndpoint,
            uvc_interfaces::{
                UVCControlInterface, UVCInterface, UVCInterfaceSubclass, UVCStreamingInterface,
            },
            UVCDescriptorTypes,
        },
        parser, USBStandardDescriptorTypes,
    },
    DescriptorDecoder, DescriptorDecoderModule, Offset, ParserError, TopologyDescriptor,
};

pub struct UVCParserModule {}

impl DescriptorDecoderModule for UVCParserModule {
    fn function_name_string(&self) -> alloc::string::String {
        "uvc".to_string()
    }

    fn filter_triple(&self, class: u8, _subclass: u8, _protocol: u8) -> bool {
        class == StandardUSBDeviceClassCode::Video as u8
        //not audio video class!
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
        let (should_be_ia, len) = USBStandardDescriptorTypes::peek_type(data)?;
        assert_eq!(
            should_be_ia,
            USBStandardDescriptorTypes::InterfaceAssociation
        );
        offset += len as usize;

        let desc_ia =
            unsafe { DescriptorDecoder::cast::<InterfaceAssociation>(&data[..offset]) }.into();
        let mut iac_level_extras: Vec<Arc<Box<dyn TopologyDescriptor>>> = Vec::new();
        let mut sub_interfaces = Vec::new();

        loop {
            match USBStandardDescriptorTypes::peek_type(&data[offset..]) {
                Ok((desc_type, _)) => {
                    if USBStandardDescriptorTypes::Interface == desc_type {
                        let interface = unsafe {
                            parser::parse_interface_group(
                                &data[offset..],
                                |slice| UVCTopology::try_parse(slice, slice[1]),
                                &mut offset,
                                "uvc".to_string(),
                            )
                        }?;
                        sub_interfaces.push(interface);
                    } else {
                        return Err(ParserError::NotSupportedDescriptorCombination(format!(
                            "{:#?} in interface association",
                            desc_type,
                        )));
                    }
                }
                Err(ParserError::PeekFailed(code)) => {
                    let got = UVCTopology::try_parse(&data[offset..], code)
                        .ok_or(ParserError::PeekFailed(code))?;
                    offset += got.actual_len();
                    iac_level_extras.push(got.into());
                }
                Err(ParserError::Ended) => {
                    return Ok((
                        TopologyUSBFunction::InterfaceAssociation(
                            desc_ia,
                            sub_interfaces,
                            iac_level_extras,
                        ),
                        offset,
                    ));
                }
                _ => return Err(crate::ParserError::NoSuitableModule),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UVCTopology {
    UVCEndpoint(UVCVideoControlInterruptEndpoint),
    UVCInterface(UVCInterface),
}

impl UVCTopology {
    pub fn try_parse(data: &[u8], code: u8) -> Option<Box<dyn TopologyDescriptor>> {
        match UVCDescriptorTypes::from_u8(code)? {
            UVCDescriptorTypes::UVCClassSpecUnderfined => todo!(),
            UVCDescriptorTypes::UVCClassSpecDevice => todo!(),
            UVCDescriptorTypes::UVCClassSpecConfiguration => todo!(),
            UVCDescriptorTypes::UVCClassSpecString => todo!(),
            UVCDescriptorTypes::UVCClassSpecInterface => {
                match UVCInterfaceSubclass::from_u8(data[2])? {
                    UVCInterfaceSubclass::UNDEFINED => panic!("impossible!"),
                    UVCInterfaceSubclass::VIDEOCONTROL => {
                        Some(Box::new(UVCTopology::UVCInterface(UVCInterface::Control(
                            UVCControlInterface::from_u8_array(data),
                        ))))
                    }
                    UVCInterfaceSubclass::VIDEOSTREAMING => {
                        Some(Box::new(UVCTopology::UVCInterface(
                            UVCInterface::Streaming(UVCStreamingInterface::from_u8_array(data)),
                        )))
                    }
                    UVCInterfaceSubclass::VIDEO_INTERFACE_COLLECTION => {
                        panic!("this subclass only appear in iac, impossible here!");
                    }
                }
            }
            UVCDescriptorTypes::UVCClassSpecVideoControlInterruptEndpoint => {
                Some(Box::new(UVCTopology::UVCEndpoint(unsafe {
                    DescriptorDecoder::cast(data)
                })))
            }
        }
    }
}

impl TopologyDescriptor for UVCTopology {
    fn desc_type(&self) -> u8 {
        match self {
            UVCTopology::UVCEndpoint(_) => {
                UVCDescriptorTypes::UVCClassSpecVideoControlInterruptEndpoint as u8
            }
            UVCTopology::UVCInterface(_) => UVCDescriptorTypes::UVCClassSpecInterface as u8,
        }
    }

    fn actual_len(&self) -> Offset {
        let ret = match self {
            UVCTopology::UVCEndpoint(uvcvideo_control_interrupt_endpoint) => {
                uvcvideo_control_interrupt_endpoint.len
            }
            UVCTopology::UVCInterface(uvcinterface) => match uvcinterface {
                UVCInterface::Control(uvccontrol_interface) => match uvccontrol_interface {
                    UVCControlInterface::Header(uvccontrol_interface_header) => {
                        uvccontrol_interface_header.length
                    }
                    UVCControlInterface::OutputTerminal(uvccontrol_interface_output_terminal) => {
                        uvccontrol_interface_output_terminal.length
                    }
                    UVCControlInterface::InputTerminal(uvccontrol_interface_input_terminal) => {
                        uvccontrol_interface_input_terminal.length
                    }
                    UVCControlInterface::ExtensionUnit(uvccontrol_interface_extension_unit) => {
                        uvccontrol_interface_extension_unit.length
                    }
                    UVCControlInterface::ProcessingUnit(uvccontrol_interface_processing_unit) => {
                        uvccontrol_interface_processing_unit.length
                    }
                },
                UVCInterface::Streaming(uvcstreaming_interface) => match uvcstreaming_interface {
                    UVCStreamingInterface::InputHeader(uvcvsinterface_input_header) => {
                        uvcvsinterface_input_header.length
                    }
                    UVCStreamingInterface::OutputHeader => todo!(),
                    UVCStreamingInterface::StillImageFrame(uvcvsinterface_still_image_frame) => {
                        uvcvsinterface_still_image_frame.length
                    }
                    UVCStreamingInterface::FormatUncompressed(
                        uvcvsinterface_format_uncompressed,
                    ) => uvcvsinterface_format_uncompressed.length,
                    UVCStreamingInterface::FrameUncompressed(uvcvsinterface_frame_uncompressed) => {
                        uvcvsinterface_frame_uncompressed.length
                    }
                    UVCStreamingInterface::FormatMjpeg(uvcvsinterface_format_mjpeg) => {
                        uvcvsinterface_format_mjpeg.length
                    }
                    UVCStreamingInterface::FrameMjpeg(uvcvsinterface_frame_mjpeg) => {
                        uvcvsinterface_frame_mjpeg.length
                    }
                    UVCStreamingInterface::FormatMpeg2ts => todo!(),
                    UVCStreamingInterface::FormatDv => todo!(),
                    UVCStreamingInterface::COLORFORMAT(uvcvsinterface_color_format) => {
                        uvcvsinterface_color_format.length
                    }
                    UVCStreamingInterface::FormatFrameBased => todo!(),
                    UVCStreamingInterface::FrameFrameBased => todo!(),
                    UVCStreamingInterface::FormatStreamBased => todo!(),
                    UVCStreamingInterface::FormatH264 => todo!(),
                    UVCStreamingInterface::FrameH264 => todo!(),
                    UVCStreamingInterface::FormatH264Simulcast => todo!(),
                    UVCStreamingInterface::FormatVp8 => todo!(),
                    UVCStreamingInterface::FrameVp8 => todo!(),
                    UVCStreamingInterface::FormatVp8Simulcast => todo!(),
                },
            },
        };
        ret as _
    }

    fn subtype(&self) -> Option<u8> {
        Some(match self {
            UVCTopology::UVCEndpoint(uvcvideo_control_interrupt_endpoint) => {
                uvcvideo_control_interrupt_endpoint.descriptor_sub_type as _
            }
            UVCTopology::UVCInterface(uvcinterface) => match uvcinterface {
                UVCInterface::Control(uvccontrol_interface) => match uvccontrol_interface {
                    UVCControlInterface::Header(uvccontrol_interface_header) => {
                        uvccontrol_interface_header.descriptor_sub_type
                    }
                    UVCControlInterface::OutputTerminal(uvccontrol_interface_output_terminal) => {
                        uvccontrol_interface_output_terminal.descriptor_sub_type
                    }
                    UVCControlInterface::InputTerminal(uvccontrol_interface_input_terminal) => {
                        uvccontrol_interface_input_terminal.descriptor_sub_type
                    }
                    UVCControlInterface::ExtensionUnit(uvccontrol_interface_extension_unit) => {
                        uvccontrol_interface_extension_unit.descriptor_sub_type
                    }
                    UVCControlInterface::ProcessingUnit(uvccontrol_interface_processing_unit) => {
                        uvccontrol_interface_processing_unit.descriptor_sub_type
                    }
                },
                UVCInterface::Streaming(uvcstreaming_interface) => match uvcstreaming_interface {
                    UVCStreamingInterface::InputHeader(uvcvsinterface_input_header) => {
                        uvcvsinterface_input_header.descriptor_sub_type
                    }
                    UVCStreamingInterface::OutputHeader => todo!(),
                    UVCStreamingInterface::StillImageFrame(uvcvsinterface_still_image_frame) => {
                        uvcvsinterface_still_image_frame.descriptor_sub_type
                    }
                    UVCStreamingInterface::FormatUncompressed(
                        uvcvsinterface_format_uncompressed,
                    ) => uvcvsinterface_format_uncompressed.descriptor_sub_type,
                    UVCStreamingInterface::FrameUncompressed(uvcvsinterface_frame_uncompressed) => {
                        uvcvsinterface_frame_uncompressed.descriptor_sub_type
                    }
                    UVCStreamingInterface::FormatMjpeg(uvcvsinterface_format_mjpeg) => {
                        uvcvsinterface_format_mjpeg.descriptor_sub_type
                    }
                    UVCStreamingInterface::FrameMjpeg(uvcvsinterface_frame_mjpeg) => {
                        uvcvsinterface_frame_mjpeg.descriptor_sub_type
                    }
                    UVCStreamingInterface::FormatMpeg2ts => todo!(),
                    UVCStreamingInterface::FormatDv => todo!(),
                    UVCStreamingInterface::COLORFORMAT(uvcvsinterface_color_format) => {
                        uvcvsinterface_color_format.descriptor_sub_type
                    }
                    UVCStreamingInterface::FormatFrameBased => todo!(),
                    UVCStreamingInterface::FrameFrameBased => todo!(),
                    UVCStreamingInterface::FormatStreamBased => todo!(),
                    UVCStreamingInterface::FormatH264 => todo!(),
                    UVCStreamingInterface::FrameH264 => todo!(),
                    UVCStreamingInterface::FormatH264Simulcast => todo!(),
                    UVCStreamingInterface::FormatVp8 => todo!(),
                    UVCStreamingInterface::FrameVp8 => todo!(),
                    UVCStreamingInterface::FormatVp8Simulcast => todo!(),
                },
            },
        })
    }
}
