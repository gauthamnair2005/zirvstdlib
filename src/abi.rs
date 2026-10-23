//! Stable binary ABI shared by the kernel, DisplayJet and future userspace.
//!
//! Every structure in this module is `#[repr(C)]` and uses fixed-width types.
//! Nothing here depends on Rust layout rules beyond the C representation.

pub const ABI_VERSION: u32 = 2;
pub const MAGIC: u64 = u64::from_le_bytes(*b"ZIRVIUM2");

pub const MAX_NAME: usize = 32;
pub const MAX_PATH: usize = 256;
pub const MAX_NODES: usize = 64;
pub const MAX_PROCESSES: usize = 1024;

/// Pixel formats understood by DisplayJet.  ARGB8888 means a little-endian
/// `u32` with bytes `B, G, R, A`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PixelFormat {
    Argb8888 = 0,
    Xrgb8888 = 1,
    Rgb565 = 2,
}

/// The framebuffer handed over by the firmware.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
    pub _reserved: u32,
}

/// A display mode or the current display state.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DisplayInfo {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub bits_per_pixel: u32,
    pub gpu_accelerated: bool,
    pub _reserved: [u8; 7],
}

/// GPU vendors recognised by the PCI probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GpuVendor {
    None = 0,
    Intel = 1,
    Amd = 2,
    Nvidia = 3,
    Virtio = 4,
    Apple = 5,
    Other = 6,
}

/// Which rendering backend is active.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GpuDriver {
    None = 0,
    /// Virtio-GPU 2D resource transfer and flush.
    VirtioGpu2D = 1,
    /// Intel GPU blitter (render engine).
    IntelBlitter = 2,
    /// CPU SIMD blits into the firmware framebuffer.
    FramebufferSimd = 3,
}

/// Information about the active GPU.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub driver: GpuDriver,
    pub vendor_id: u16,
    pub device_id: u16,
    pub mmio_base: u64,
    pub mmio_size: u64,
    pub accelerated: bool,
    pub _reserved: [u8; 7],
    pub name: [u8; 64],
}

impl GpuInfo {
    pub fn name_str(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..end]).unwrap_or("unknown GPU")
    }
}

/// Input event kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum InputKind {
    KeyDown = 1,
    KeyUp = 2,
    PointerMove = 3,
    PointerDown = 4,
    PointerUp = 5,
    Scroll = 6,
    Resize = 7,
    Focus = 8,
}

/// Modifier bits carried by every input event.
pub mod modifiers {
    pub const SHIFT: u16 = 1 << 0;
    pub const CTRL: u16 = 1 << 1;
    pub const ALT: u16 = 1 << 2;
    pub const SUPER: u16 = 1 << 3;
    pub const CAPS_LOCK: u16 = 1 << 4;
    pub const NUM_LOCK: u16 = 1 << 5;
}

/// Mouse button bits.
pub mod buttons {
    pub const LEFT: u16 = 1 << 0;
    pub const RIGHT: u16 = 1 << 1;
    pub const MIDDLE: u16 = 1 << 2;
    pub const BACK: u16 = 1 << 3;
    pub const FORWARD: u16 = 1 << 4;
}

/// Non-printable key codes live in the Unicode private-use area so that
/// printable keys can simply carry their code point.
pub mod key {
    pub const NONE: u32 = 0;
    pub const ESCAPE: u32 = 0xE000;
    pub const TAB: u32 = 0xE001;
    pub const BACKSPACE: u32 = 0xE002;
    pub const ENTER: u32 = 0xE003;
    pub const DELETE: u32 = 0xE004;
    pub const INSERT: u32 = 0xE005;
    pub const HOME: u32 = 0xE006;
    pub const END: u32 = 0xE007;
    pub const PAGE_UP: u32 = 0xE008;
    pub const PAGE_DOWN: u32 = 0xE009;
    pub const LEFT: u32 = 0xE00A;
    pub const RIGHT: u32 = 0xE00B;
    pub const UP: u32 = 0xE00C;
    pub const DOWN: u32 = 0xE00D;
    pub const F1: u32 = 0xE010;
    pub const F2: u32 = 0xE011;
    pub const F3: u32 = 0xE012;
    pub const F4: u32 = 0xE013;
    pub const F5: u32 = 0xE014;
    pub const F6: u32 = 0xE015;
    pub const F7: u32 = 0xE016;
    pub const F8: u32 = 0xE017;
    pub const F9: u32 = 0xE018;
    pub const F10: u32 = 0xE019;
    pub const F11: u32 = 0xE01A;
    pub const F12: u32 = 0xE01B;
    pub const SUPER: u32 = 0xE020;
    pub const MENU: u32 = 0xE021;
    pub const CAPS_LOCK: u32 = 0xE022;
    pub const NUM_LOCK: u32 = 0xE023;
    pub const SCROLL_LOCK: u32 = 0xE024;
    pub const PRINT_SCREEN: u32 = 0xE025;
    pub const PAUSE: u32 = 0xE026;
}

/// A single input event.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InputEvent {
    pub kind: InputKind,
    pub modifiers: u16,
    pub buttons: u16,
    pub code: u32,
    pub value: i32,
    pub x: i32,
    pub y: i32,
}

impl InputEvent {
    pub const fn key(kind: InputKind, code: u32, modifiers: u16) -> Self {
        Self {
            kind,
            modifiers,
            buttons: 0,
            code,
            value: 0,
            x: 0,
            y: 0,
        }
    }

    pub const fn pointer(kind: InputKind, x: i32, y: i32, buttons: u16) -> Self {
        Self {
            kind,
            modifiers: 0,
            buttons,
            code: 0,
            value: 0,
            x,
            y,
        }
    }
}

/// MOSIX node state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum NodeState {
    Offline = 0,
    Booting = 1,
    Online = 2,
    Busy = 3,
    Draining = 4,
}

/// A MOSIX cluster member.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct NodeInfo {
    pub id: u32,
    pub state: NodeState,
    pub cpu_count: u32,
    pub load_permille: u32,
    pub memory_mib: u32,
    pub _reserved: u32,
    pub name: [u8; MAX_NAME],
}

impl NodeInfo {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            state: NodeState::Offline,
            cpu_count: 0,
            load_permille: 0,
            memory_mib: 0,
            _reserved: 0,
            name: [0; MAX_NAME],
        }
    }

    pub fn name_str(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..end]).unwrap_or("node")
    }
}

/// Process state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ProcessState {
    Unused = 0,
    Ready = 1,
    Running = 2,
    Sleeping = 3,
    Blocked = 4,
    Zombie = 5,
    Migrating = 6,
}

/// A process visible to the MOSIX scheduler and the system monitor.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: u32,
    pub state: ProcessState,
    pub priority: u32,
    pub cpu_ticks: u64,
    pub memory_kib: u64,
    pub home_node: u32,
    pub current_node: u32,
    pub name: [u8; MAX_NAME],
}

impl ProcessInfo {
    pub const fn empty() -> Self {
        Self {
            pid: 0,
            parent_pid: 0,
            state: ProcessState::Unused,
            priority: 0,
            cpu_ticks: 0,
            memory_kib: 0,
            home_node: 0,
            current_node: 0,
            name: [0; MAX_NAME],
        }
    }

    pub fn name_str(&self) -> &str {
        let end = self
            .name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..end]).unwrap_or("process")
    }
}

/// Fill a fixed-size name field from a string, truncating at a UTF-8
/// boundary.
pub fn set_name<const N: usize>(field: &mut [u8; N], name: &str) {
    field.fill(0);
    let mut count = 0;
    for character in name.chars() {
        let length = character.len_utf8();
        if count + length >= N {
            break;
        }
        character.encode_utf8(&mut field[count..]);
        count += length;
    }
}

/// The Zirvium syscall interface.  `int 0x80` with the number in `rax` and
/// arguments in `rdi`, `rsi` and `rdx`; the result is returned in `rax`.
pub mod syscall {
    pub const PING: u64 = 0;
    pub const WRITE: u64 = 1;
    pub const GETPID: u64 = 2;
    pub const UPTIME_MS: u64 = 3;
    pub const NODE_COUNT: u64 = 4;
    pub const SPAWN: u64 = 5;
    pub const MIGRATE: u64 = 6;
    pub const MEMORY_FREE_KIB: u64 = 7;
    pub const GPU_VENDOR: u64 = 8;
    pub const PROCESS_COUNT: u64 = 9;
    pub const LOAD_PERMILLE: u64 = 10;
    pub const TICKS: u64 = 11;

    /// Returned for an unknown syscall number.
    pub const UNKNOWN: u64 = u64::MAX;
    /// Returned by [`PING`].
    pub const PONG: u64 = 0x5A49_5256; // "ZIRV"
}
