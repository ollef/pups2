use crate::fifo::Fifo;

use super::bus::Bytes;

pub struct Gif {
    fifo: Fifo<u128>,
    control: u32,                       // CTRL
    mode: u32,                          // MODE
    status: u32,                        // STAT
    tag: [u32; 4],                      // TAG0, TAG1, TAG2, TAG3
    transfer_status_counter: u32,       // CNT
    path3_transfer_status_counter: u32, // P3CNT
    path3_tag_value: u32,               // P3TAG
}

impl Gif {
    pub fn new() -> Gif {
        Gif {
            fifo: Fifo::with_capacity(16),
            control: 0,
            mode: 0,
            status: 0,
            tag: [0; 4],
            transfer_status_counter: 0,
            path3_transfer_status_counter: 0,
            path3_tag_value: 0,
        }
    }

    pub fn write<T: Bytes>(&mut self, address: u32, value: T) {
        assert!(std::mem::size_of::<T>() == 4);
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        let value = u32::from_bytes(value.to_bytes().as_ref());
        match address {
            0x1000_3000 => self.control = value,
            0x1000_3010 => self.mode = value,
            0x1000_3020 => self.status = value,
            0x1000_3040 => self.tag[0] = value,
            0x1000_3050 => self.tag[1] = value,
            0x1000_3060 => self.tag[2] = value,
            0x1000_3070 => self.tag[3] = value,
            0x1000_3080 => self.transfer_status_counter = value,
            0x1000_3090 => self.path3_transfer_status_counter = value,
            0x1000_30a0 => self.path3_tag_value = value,
            _ => panic!(
                "Invalid GIF write of {} at address: 0x{:08X}",
                value, address
            ),
        }
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        assert!(std::mem::size_of::<T>() == 4);
        assert!(address & (std::mem::size_of::<T>() - 1) as u32 == 0);
        match address {
            0x1000_3000 => T::from_bytes(&self.control.to_bytes()),
            0x1000_3010 => T::from_bytes(&self.mode.to_bytes()),
            0x1000_3020 => T::from_bytes(&self.status.to_bytes()),
            0x1000_3040 => T::from_bytes(&self.tag[0].to_bytes()),
            0x1000_3050 => T::from_bytes(&self.tag[1].to_bytes()),
            0x1000_3060 => T::from_bytes(&self.tag[2].to_bytes()),
            0x1000_3070 => T::from_bytes(&self.tag[3].to_bytes()),
            0x1000_3080 => T::from_bytes(&self.transfer_status_counter.to_bytes()),
            0x1000_3090 => T::from_bytes(&self.path3_transfer_status_counter.to_bytes()),
            0x1000_30a0 => T::from_bytes(&self.path3_tag_value.to_bytes()),
            _ => panic!("Invalid GIF read at address: 0x{:08X}", address),
        }
    }
}
