use std::cell::RefCell;

use crate::{bits::Bits, bytes::Bytes};

#[derive(Debug, Default, Clone)]
pub struct Rdram {
    mch_ricm: u32,
    mch_drd: u32,
    sdevid: RefCell<u32>,
}

impl Rdram {
    pub fn write<T: Bytes>(&mut self, address: u32, value: T) {
        match std::mem::size_of::<T>() {
            4 => self.write32(address, u32::from_bytes(value.to_bytes().as_ref())),
            _ => panic!("Invalid write size {}", std::mem::size_of::<T>()),
        }
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        match std::mem::size_of::<T>() {
            4 => T::from_bytes(self.read32(address).to_bytes().as_ref()),
            _ => panic!("Invalid read size {}", std::mem::size_of::<T>()),
        }
    }

    pub fn write32(&mut self, address: u32, value: u32) {
        match address {
            0x1000_F430 => {
                let sa = value.bits(16..24);
                let sbc = value.bits(6..10);

                if sa == 0x21 && sbc == 0x1 && !self.mch_drd.bit(7) {
                    *self.sdevid.borrow_mut() = 0;
                }

                self.mch_ricm = value.bits(0..31);
            }
            0x1000_F440 => self.mch_drd = value,
            _ => panic!(
                "Invalid RDRAM write of {} at address: 0x{:08x}",
                value, address
            ),
        }
    }

    pub fn read32(&self, address: u32) -> u32 {
        match address {
            0x1000_F430 => 0,
            0x1000_F440 => {
                let sop = self.mch_ricm.bits(6..10);
                if sop == 0 {
                    let sa = self.mch_ricm.bits(16..24);
                    match sa {
                        0x21 => {
                            if *self.sdevid.borrow() < 2 {
                                *self.sdevid.borrow_mut() += 1;
                                return 0x1F;
                            }
                        }
                        0x23 => {
                            return 0x0D0D;
                        }
                        0x24 => {
                            return 0x0090;
                        }
                        0x40 => {
                            return self.mch_ricm.bits(0..5);
                        }
                        _ => {}
                    }
                }
                return 0;
            }
            _ => panic!("Invalid RDRAM read at address: 0x{:08x}", address),
        }
    }
}
