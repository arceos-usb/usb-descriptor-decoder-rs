use alloc::string::ToString;

use crate::{descriptors::desc_device::StandardUSBDeviceClassCode, DescriptorDecoderModule};

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
        todo!()
    }
}
