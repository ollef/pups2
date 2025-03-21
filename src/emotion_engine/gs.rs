use crate::bytes::Bytes;

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pcrtc_mode: u64,            // PMODE
    sync_mode1: u64,            // SMODE1
    sync_mode2: u64,            // SMODE2
    dram_refresh: u64,          // SRFSH
    synch1: u64,                // SYNCH1
    synch2: u64,                // SYNCH2
    syncv: u64,                 // SYNCV
    display_frame_buffer1: u64, // DISPFB1
    display1: u64,              // DISPLAY1
    display_frame_buffer2: u64, // DISPFB1
    display2: u64,              // DISPLAY1
    write_buffer: u64,          // EXTBUF
    write_data: u64,            // EXTDATA
    write_start: u64,           // EXTWRITE
    background_color: u64,      // BGCOLOR
    status: u64,                // CSR
    interrupt_mask: u64,        // IMR
    bus_direction: u64,         // BUSDIR
    signal_label_id: u64,       // SIGLBLID
}

impl Gs {
    pub fn new() -> Gs {
        Gs {
            local_memory: vec![0; LOCAL_MEMORY_SIZE].into_boxed_slice(),
            pcrtc_mode: 0,
            sync_mode1: 0,
            sync_mode2: 0,
            dram_refresh: 0,
            synch1: 0,
            synch2: 0,
            syncv: 0,
            display_frame_buffer1: 0,
            display1: 0,
            display_frame_buffer2: 0,
            display2: 0,
            write_buffer: 0,
            write_data: 0,
            write_start: 0,
            background_color: 0,
            status: 0,
            interrupt_mask: 0,
            bus_direction: 0,
            signal_label_id: 0,
        }
    }

    pub fn write<T: Bytes>(&mut self, address: u32, value: T) {
        assert!(std::mem::size_of::<T>() == 8);
        let value = u64::from_bytes(value.to_bytes().as_ref());
        match address {
            0x1200_0000 => self.pcrtc_mode = value,
            0x1200_0010 => self.sync_mode1 = value,
            0x1200_0020 => self.sync_mode2 = value,
            0x1200_0030 => self.dram_refresh = value,
            0x1200_0040 => self.synch1 = value,
            0x1200_0050 => self.synch2 = value,
            0x1200_0060 => self.syncv = value,
            0x1200_0070 => self.display_frame_buffer1 = value,
            0x1200_0080 => self.display1 = value,
            0x1200_0090 => self.display_frame_buffer2 = value,
            0x1200_00A0 => self.display2 = value,
            0x1200_00B0 => self.write_buffer = value,
            0x1200_00C0 => self.write_data = value,
            0x1200_00D0 => self.write_start = value,
            0x1200_00E0 => self.background_color = value,
            0x1200_1000 => self.status = value,
            0x1200_1010 => self.interrupt_mask = value,
            0x1200_1040 => self.bus_direction = value,
            0x1200_1080 => self.signal_label_id = value,
            _ => panic!("Invalid GS write64 {} to address: 0x{:08X}", value, address),
        }
    }

    pub fn read<T: Bytes>(&self, address: u32) -> T {
        assert!(std::mem::size_of::<T>() == 8);
        match address {
            0x1200_0000 => T::from_bytes(&self.pcrtc_mode.to_bytes()),
            0x1200_0010 => T::from_bytes(&self.sync_mode1.to_bytes()),
            0x1200_0020 => T::from_bytes(&self.sync_mode2.to_bytes()),
            0x1200_0030 => T::from_bytes(&self.dram_refresh.to_bytes()),
            0x1200_0040 => T::from_bytes(&self.synch1.to_bytes()),
            0x1200_0050 => T::from_bytes(&self.synch2.to_bytes()),
            0x1200_0060 => T::from_bytes(&self.syncv.to_bytes()),
            0x1200_0070 => T::from_bytes(&self.display_frame_buffer1.to_bytes()),
            0x1200_0080 => T::from_bytes(&self.display1.to_bytes()),
            0x1200_0090 => T::from_bytes(&self.display_frame_buffer2.to_bytes()),
            0x1200_00A0 => T::from_bytes(&self.display2.to_bytes()),
            0x1200_00B0 => T::from_bytes(&self.write_buffer.to_bytes()),
            0x1200_00C0 => T::from_bytes(&self.write_data.to_bytes()),
            0x1200_00D0 => T::from_bytes(&self.write_start.to_bytes()),
            0x1200_00E0 => T::from_bytes(&self.background_color.to_bytes()),
            0x1200_1000 => T::from_bytes(&self.status.to_bytes()),
            0x1200_1010 => T::from_bytes(&self.interrupt_mask.to_bytes()),
            0x1200_1040 => T::from_bytes(&self.bus_direction.to_bytes()),
            0x1200_1080 => T::from_bytes(&self.signal_label_id.to_bytes()),
            _ => panic!("Invalid GS read64 from address: 0x{:08X}", address),
        }
    }
}
