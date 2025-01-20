use alloc::vec;
use alloc::vec::Vec;

use super::{
    desc_configuration::Configuration,
    desc_device::Device,
    desc_endpoint::Endpoint,
    desc_interface::{Interface, InterfaceAssociation},
    desc_uvc::uvc_endpoints::UVCVideoControlInterruptEndpoint,
    parser::ParserMetaData,
    USBDescriptor,
};

#[derive(Clone, Debug)]
pub struct TopologicalUSBDescriptorDevice {
    pub data: Device,
    pub child: Vec<TopologicalUSBDescriptorConfiguration>,
}

#[derive(Clone, Debug)]
pub struct TopologicalUSBDescriptorConfiguration {
    pub data: Configuration,
    pub child: Vec<TopologicalUSBDescriptorFunction>,
}

#[derive(Clone, Debug)]
pub enum TopologicalUSBDescriptorFunction {
    InterfaceAssociation((InterfaceAssociation, Vec<TopologicalUSBDescriptorFunction>)), //maybe we would have multi layer compose device in future? for now just treat it as a trick!
    Interface(
        Vec<(
            Interface,
            Vec<USBDescriptor>,
            Vec<TopologicalUSBDescriptorEndpoint>,
        )>,
    ),
}
#[derive(Clone, Debug)]
pub struct TopologicalUSBDescriptorRoot {
    pub device: TopologicalUSBDescriptorDevice,
    pub others: Vec<USBDescriptor>,
    pub metadata: ParserMetaData,
}

#[derive(Clone, Debug)]
pub enum TopologicalUSBDescriptorEndpoint {
    Standard(Endpoint),
    UNVVideoControlInterruptEndpoint(UVCVideoControlInterruptEndpoint),
}

pub enum USBFunctionExpressions<'a> {
    Interface(&'a Interface),
    InterfaceAssociation(&'a InterfaceAssociation),
    Device(&'a Device),
}

impl TopologicalUSBDescriptorRoot {
    pub fn interfaces<'a>(&'a self) -> Vec<USBFunctionExpressions<'a>> {
        if self.device.data.is_refer_interface() {
            self.device
                .child
                .first()
                .expect("atleast 1 cfg, this device got some issue!")
                .child
                .iter()
                .map(|int| match int {
                    TopologicalUSBDescriptorFunction::InterfaceAssociation((ia, _)) => {
                        USBFunctionExpressions::InterfaceAssociation(ia)
                    }
                    TopologicalUSBDescriptorFunction::Interface(vec) => {
                        USBFunctionExpressions::Interface(
                            &vec.get(0)
                                .expect(
                                    "atleast 1 interface exist, this device must had some issue!",
                                )
                                .0,
                        )
                    }
                })
                .collect()
        } else {
            vec![USBFunctionExpressions::Device(&self.device.data)]
        }
    }

    pub fn configs<'a>(&'a self) -> Vec<&'a Configuration> {
        self.device.child.iter().map(|cfg| &cfg.data).collect()
    }
}

impl<'a> USBFunctionExpressions<'a> {
    pub fn class_subclass_protocol(&self) -> (u8, u8, u8) {
        match self {
            USBFunctionExpressions::Interface(interface) => (
                interface.interface_class,
                interface.interface_subclass,
                interface.interface_protocol,
            ),
            USBFunctionExpressions::InterfaceAssociation(interface_association) => (
                interface_association.function_class,
                interface_association.function_subclass,
                interface_association.function_protocol,
            ),
            USBFunctionExpressions::Device(device) => {
                (device.class, device.subclass, device.protocol)
            }
        }
    }
}
