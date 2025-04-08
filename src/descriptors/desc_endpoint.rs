use bit_field::BitField;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use super::PortSpeed;

#[derive(Copy, Clone, Default, Debug)]
#[repr(C, packed)]
pub struct Endpoint {
    len: u8,
    descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
    pub ssc: Option<SuperSpeedCmp>,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct SuperSpeedCmp {
    pub kind: u8,
    pub max_burst: u8,
    pub attributes: u8,
    pub bytes_per_interval: u16,
}

impl Endpoint {
    pub fn endpoint_type(&self) -> EndpointType {
        EndpointType::from_u8(if self.attributes == 0 {
            4
        } else {
            self.attributes.get_bits(0..=1)
                + if self.endpoint_address.get_bit(7) {
                    4
                } else {
                    0
                }
        })
        .expect("EndpointType must be convertible from `attributes` and `endpoint_address`.")
    }

    pub fn calc_actual_interval(&self, port_speed: PortSpeed) -> u8 {
        if let PortSpeed::FullSpeed | PortSpeed::LowSpeed = port_speed {
            if let EndpointType::IsochOut | EndpointType::IsochIn = self.endpoint_type() {
                self.interval + 2
            } else {
                self.interval + 3
            }
        } else {
            self.interval - 1
        }
    }

    pub fn max_streams(&self) -> Option<u8> {
        //TODO: complete me
        if self.is_bulk_out() {
            Some(self.calculate_max_streams())
        } else {
            None
        }
    }

    pub fn is_bulk_out(&self) -> bool {
        self.endpoint_type() == EndpointType::BulkOut
    }

    pub fn calculate_max_streams(&self) -> u8 {
        self.ssc
            .as_ref()
            .map(|ssc| {
                if self.is_bulk_out() {
                    let raw = ssc.attributes & 0x1F;
                    raw
                } else {
                    0
                }
            })
            .unwrap()
    }

    pub fn is_superspeedplus(&self) -> bool {
        false
    }

    pub fn mult(&self, lec: bool) -> u8 {
        if !lec && self.endpoint_type() == EndpointType::IsochOut {
            if self.is_superspeedplus() {
                return 0;
            }
            self.ssc
                .as_ref()
                .map(|ssc| ssc.attributes & 0x3)
                .unwrap_or(0)
        } else {
            0
        }
    }

    pub fn doorbell_value_aka_dci(&self) -> u32 {
        2 * u32::from(self.endpoint_address.get_bits(0..=3))
            + self.endpoint_address.get_bit(7) as u32
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive)]
pub enum EndpointType {
    /// Not Valid.
    NotValid = 0,
    /// Isoch Out.
    IsochOut = 1,
    /// Bulk Out.
    BulkOut = 2,
    /// Interrupt Out.
    InterruptOut = 3,
    /// Control Bidirectional.
    Control = 4,
    /// Isoch In.
    IsochIn = 5,
    /// Bulk In.
    BulkIn = 6,
    /// Interrupt In.
    InterruptIn = 7,
}

impl EndpointType {
    pub fn cast<T>(&self) -> T
    where
        T: FromPrimitive,
    {
        FromPrimitive::from_u8(self.to_u8().unwrap()).unwrap()
    }
}
