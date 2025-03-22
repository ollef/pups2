use std::collections::VecDeque;

use num_derive::FromPrimitive;

use crate::bytes::Bytes;

const LOCAL_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct Gs {
    local_memory: Box<[u8]>,
    pub command_queue: VecDeque<(Register, u64)>,
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
            command_queue: VecDeque::new(),
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

    pub fn write_privileged<T: Bytes>(&mut self, address: u32, value: T) {
        match std::mem::size_of::<T>() {
            8 => self.write_privileged64(address, u64::from_bytes(value.to_bytes().as_ref())),
            _ => panic!("Invalid GS write size: {}", std::mem::size_of::<T>()),
        }
    }
    pub fn read_privileged<T: Bytes>(&self, address: u32) -> T {
        match std::mem::size_of::<T>() {
            8 => T::from_bytes(self.read_privileged64(address).to_bytes().as_ref()),
            _ => panic!("Invalid GS read size: {}", std::mem::size_of::<T>()),
        }
    }

    pub fn write_privileged64(&mut self, address: u32, value: u64) {
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

    pub fn read_privileged64(&self, address: u32) -> u64 {
        match address {
            0x1200_0000 => self.pcrtc_mode,
            0x1200_0010 => self.sync_mode1,
            0x1200_0020 => self.sync_mode2,
            0x1200_0030 => self.dram_refresh,
            0x1200_0040 => self.synch1,
            0x1200_0050 => self.synch2,
            0x1200_0060 => self.syncv,
            0x1200_0070 => self.display_frame_buffer1,
            0x1200_0080 => self.display1,
            0x1200_0090 => self.display_frame_buffer2,
            0x1200_00A0 => self.display2,
            0x1200_00B0 => self.write_buffer,
            0x1200_00C0 => self.write_data,
            0x1200_00D0 => self.write_start,
            0x1200_00E0 => self.background_color,
            0x1200_1000 => self.status,
            0x1200_1010 => self.interrupt_mask,
            0x1200_1040 => self.bus_direction,
            0x1200_1080 => self.signal_label_id,
            _ => panic!("Invalid GS read64 from address: 0x{:08X}", address),
        }
    }

    pub fn step(&mut self) {
        while let Some((register, data)) = self.command_queue.pop_front() {
            println!("Command: {:?}={:?}", register, data)
        }
    }
}

#[repr(u8)]
#[derive(FromPrimitive, Debug)]
pub enum Register {
    Primitive = 0x00,             // PRIM Drawing primitive setting
    Rgbaq = 0x01,                 // RGBAQ Vertex color setting
    St = 0x02,                    // ST Vertex texture coordinate setting (texture coordinates)
    Uv = 0x03,                    // UV Vertex texture coordinate setting (texel coordinates)
    Xyzf2 = 0x04,                 // XYZF2 Vertex coordinate value setting
    Xyz2 = 0x05,                  // XYZ2 Vertex coordinate value setting
    Tex0_1 = 0x06,                // TEX0_1 Texture information setting
    Tex0_2 = 0x07,                // TEX0_2 Texture information setting
    Clamp1 = 0x08,                // CLAMP_1 Texture wrap mode
    Clamp2 = 0x09,                // CLAMP_2 Texture wrap mode
    Fog = 0x0a,                   // FOG Vertex fog value setting
    Xyzf3 = 0x0c,                 // XYZF3 Vertex coordinate value setting (without drawing kick)
    Xyz3 = 0x0d,                  // XYZ3 Vertex coordinate value setting (without drawing kick)
    Texture1_1 = 0x14,            // TEX1_1 Texture information setting
    Texture1_2 = 0x15,            // TEX1_2 Texture information setting
    Texture2_1 = 0x16,            // TEX2_1 Texture information setting
    Texture2_2 = 0x17,            // TEX2_2 Texture information setting
    XyOffset1 = 0x18,             // XYOFFSET_1 Offset value setting
    XyOffset2 = 0x19,             // XYOFFSET_2 Offset value setting
    PrimitiveModeControl = 0x1a,  // PRMODECONT Specification of primitive attribute setting method
    PrimitiveMode = 0x1b,         // PRMODE Drawing primitive attribute setting
    TexClut = 0x1c,               // TEXCLUT CLUT position setting
    ScanMask = 0x22,              // SCANMSK Raster address mask setting
    MipMap1_1 = 0x34,             // MIPTBP1_1 MIPMAP information setting (Level 1 ñ 3)
    MipMap1_2 = 0x35,             // MIPTBP1_2 MIPMAP information setting (Level 1 ñ 3)
    MipMap2_1 = 0x36,             // MIPTBP2_1 MIPMAP information setting (Level 4 ñ 6)
    MipMap2_2 = 0x37,             // MIPTBP2_2 MIPMAP information setting (Level 4 ñ 6)
    TextureAlpha = 0x3b,          // TEXA Texture alpha value setting
    FogColor = 0x3d,              // FOGCOL Distant fog color setting
    TextureFlush = 0x3f,          // TEXFLUSH Texture page buffer disabling
    Scissor1 = 0x40,              // SCISSOR_1 Scissoring area setting
    Scissor2 = 0x41,              // SCISSOR_2 Scissoring area setting
    Alpha1 = 0x42,                // ALPHA_1 Alpha blending setting
    Alpha2 = 0x43,                // ALPHA_2 Alpha blending setting
    DitherMatrix = 0x44,          // DIMX Dither matrix setting
    DitherControl = 0x45,         // DTHE Dither control
    ColorClamp = 0x46,            // COLCLAMP Color clamp control
    PixelTest1 = 0x47,            // TEST_1 Pixel test control
    PixelTest2 = 0x48,            // TEST_2 Pixel test control
    PixelAlphaBlending = 0x49,    // PABE Alpha blending control in pixel units
    FrameBufferAlpha1 = 0x4a,     // FBA_1 Alpha correction value
    FrameBufferAlpha2 = 0x4b,     // FBA_2 Alpha correction value
    FrameBuffer1 = 0x4c,          // FRAME_1 Frame buffer setting
    FrameBuffer2 = 0x4d,          // FRAME_2 Frame buffer setting
    ZBuffer1 = 0x4e,              // ZBUF_1 Z buffer setting
    ZBuffer2 = 0x4f,              // ZBUF_2 Z buffer setting
    BitBlitBuffer = 0x50,         // BITBLTBUF Setting for transmission between buffers
    TransmissionPosition = 0x51,  // TRXPOS Specification for transmission area in buffers
    TransmissionSize = 0x52,      // TRXREG Specification for transmission area in buffers
    TransmissionDirection = 0x53, // TRXDIR Activation of transmission between buffers
    TransmissionData = 0x54,      // HWREG Data port for transmission between buffers
    SignalSignal = 0x60,          // SIGNAL SIGNAL event occurrence request
    SignalFinish = 0x61,          // FINISH FINISH event occurrence request
    SignalLabel = 0x62,           // LABEL LABEL event occurrence request
}
