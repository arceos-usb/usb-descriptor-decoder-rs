use alloc::boxed::Box;

use crate::DescriptorDecoder;
#[cfg(feature = "hid")]
mod hid;
#[cfg(feature = "uvc")]
mod uvc;

pub fn load_embed_parsers(mut decoder: DescriptorDecoder) -> DescriptorDecoder {
    #[cfg(feature = "hid")]
    {
        decoder.add_module(Box::new(hid::HIDParserModule {}));
    }
    #[cfg(feature = "uvc")]
    {
        decoder.add_module(Box::new(uvc::UVCParserModule {}));
    }

    decoder
}
