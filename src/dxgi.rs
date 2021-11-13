use crate::{CommandQueue, Resource, SampleDesc};
use std::ptr;
use windows::{
    runtime::{self, Interface},
    Win32::{
        Foundation::{self, HWND},
        Graphics::Dxgi::{self, IDXGIDevice2},
    },
};

bitflags! {
    pub struct FactoryCreationFlags: u32 {
        const DEBUG = Dxgi::DXGI_CREATE_FACTORY_DEBUG;
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum Scaling {
    Stretch = Dxgi::DXGI_SCALING_STRETCH.0 as u32,
    Identity = Dxgi::DXGI_SCALING_NONE.0 as u32,
    Aspect = Dxgi::DXGI_SCALING_ASPECT_RATIO_STRETCH.0 as u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum SwapEffect {
    Discard = Dxgi::DXGI_SWAP_EFFECT_DISCARD.0 as u32,
    Sequential = Dxgi::DXGI_SWAP_EFFECT_SEQUENTIAL.0 as u32,
    FlipDiscard = Dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD.0 as u32,
    FlipSequential = Dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL.0 as u32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum AlphaMode {
    Unspecified = Dxgi::DXGI_ALPHA_MODE_UNSPECIFIED.0 as u32,
    Premultiplied = Dxgi::DXGI_ALPHA_MODE_PREMULTIPLIED.0 as u32,
    Straight = Dxgi::DXGI_ALPHA_MODE_STRAIGHT.0 as u32,
    Ignore = Dxgi::DXGI_ALPHA_MODE_IGNORE.0 as u32,
    ForceDword = Dxgi::DXGI_ALPHA_MODE_FORCE_DWORD.0 as u32,
}

bitflags! {
    pub struct DxgiUsage: u32 {
        const BACKBUFFER = Dxgi::DXGI_USAGE_BACK_BUFFER;
        const DISCARDONPRESENT = Dxgi::DXGI_USAGE_DISCARD_ON_PRESENT;
        const READONLY = Dxgi::DXGI_USAGE_READ_ONLY;
        const RENDERTARGETOUTPUT = Dxgi::DXGI_USAGE_RENDER_TARGET_OUTPUT;
        const SHADERINPUT = Dxgi::DXGI_USAGE_SHADER_INPUT;
        const SHARED = Dxgi::DXGI_USAGE_SHARED;
        const UNORDEREDACCESS = Dxgi::DXGI_USAGE_UNORDERED_ACCESS;
    }
}

pub type Adapter1 = Dxgi::IDXGIAdapter1;
pub type Factory2 = Dxgi::IDXGIFactory2;
pub type Factory4 = Dxgi::IDXGIFactory4;
pub type InfoQueue = Dxgi::IDXGIInfoQueue;
pub type SwapChain = Dxgi::IDXGISwapChain;
pub type SwapChain1 = Dxgi::IDXGISwapChain1;
pub type SwapChain3 = Dxgi::IDXGISwapChain3;

#[cfg(feature = "libloading")]
#[derive(Debug)]
pub struct DxgiLib {
    lib: libloading::Library,
}

#[cfg(feature = "libloading")]
impl DxgiLib {
    pub fn new() -> Result<Self, libloading::Error> {
        unsafe { libloading::Library::new("dxgi.dll").map(|lib| DxgiLib { lib }) }
    }

    pub fn create_factory2(&self, flags: FactoryCreationFlags) -> runtime::Result<Factory4> {
        unsafe { Dxgi::CreateDXGIFactory2::<Factory4>(flags.bits()) }
    }

    pub fn get_debug_interface1(&self) -> Result<runtime::Result<InfoQueue>, libloading::Error> {
        let hr = unsafe { Dxgi::DXGIGetDebugInterface1::<InfoQueue>(0) };
        match hr {
            Ok(queue) => Ok(Ok(queue)),
            Err(err) => Ok(runtime::Result::Err(runtime::Error::new(
                err.code(),
                err.message().as_str(),
            ))),
        }
    }
}

// TODO: strong types
pub struct SwapchainDesc {
    pub width: u32,
    pub height: u32,
    pub format: Dxgi::DXGI_FORMAT,
    pub stereo: bool,
    pub sample: SampleDesc,
    pub buffer_usage: DxgiUsage,
    pub buffer_count: u32,
    pub scaling: Scaling,
    pub swap_effect: SwapEffect,
    pub alpha_mode: AlphaMode,
    pub flags: u32,
}

pub trait IFactory2 {
    fn create_swapchain_for_hwnd(
        &self,
        queue: CommandQueue,
        hwnd: HWND,
        desc: &SwapchainDesc,
    ) -> runtime::Result<SwapChain1>;
}

impl IFactory2 for Factory2 {
    // TODO: interface not complete
    fn create_swapchain_for_hwnd(
        &self,
        queue: CommandQueue,
        hwnd: HWND,
        desc: &SwapchainDesc,
    ) -> runtime::Result<SwapChain1> {
        let desc = Dxgi::DXGI_SWAP_CHAIN_DESC1 {
            AlphaMode: Dxgi::DXGI_ALPHA_MODE(desc.alpha_mode as _),
            BufferCount: desc.buffer_count,
            Width: desc.width,
            Height: desc.height,
            Format: desc.format,
            Flags: desc.flags,
            BufferUsage: desc.buffer_usage.bits(),
            SampleDesc: Dxgi::DXGI_SAMPLE_DESC {
                Count: desc.sample.count,
                Quality: desc.sample.quality,
            },
            Scaling: Dxgi::DXGI_SCALING(desc.scaling as _),
            Stereo: Foundation::BOOL(desc.stereo as _),
            SwapEffect: Dxgi::DXGI_SWAP_EFFECT(desc.swap_effect as _),
        };

        unsafe {
            let mut device: Option<IDXGIDevice2> = None;
            queue.GetDevice(&mut device)?;
            let device = device.unwrap();
            self.CreateSwapChainForHwnd(device, hwnd, &desc, ptr::null(), None)
        }
    }
}

pub trait IFactory4 {
    #[cfg(feature = "implicit-link")]
    fn create(flags: FactoryCreationFlags) -> runtime::Result<Factory4>;

    fn as_factory2(&self) -> Factory2;
    fn enumerate_adapters(&self, id: u32) -> runtime::Result<Adapter1>;
}

impl IFactory4 for Factory4 {
    #[cfg(feature = "implicit-link")]
    fn create(flags: FactoryCreationFlags) -> runtime::Result<Self> {
        unsafe { Dxgi::CreateDXGIFactory2::<Self>(flags.bits()) }
    }

    fn as_factory2(&self) -> Factory2 {
        self.cast::<Factory2>().unwrap()
    }

    fn enumerate_adapters(&self, id: u32) -> runtime::Result<Adapter1> {
        unsafe { self.EnumAdapters1(id) }
    }
}

bitflags! {
    pub struct SwapChainPresentFlags: u32 {
        const DXGI_PRESENT_DO_NOT_SEQUENCE = Dxgi::DXGI_PRESENT_DO_NOT_SEQUENCE;
        const DXGI_PRESENT_TEST = Dxgi::DXGI_PRESENT_TEST;
        const DXGI_PRESENT_RESTART = Dxgi::DXGI_PRESENT_RESTART;
        const DXGI_PRESENT_DO_NOT_WAIT = Dxgi::DXGI_PRESENT_DO_NOT_WAIT;
        const DXGI_PRESENT_RESTRICT_TO_OUTPUT = Dxgi::DXGI_PRESENT_RESTRICT_TO_OUTPUT;
        const DXGI_PRESENT_STEREO_PREFER_RIGHT = Dxgi::DXGI_PRESENT_STEREO_PREFER_RIGHT;
        const DXGI_PRESENT_STEREO_TEMPORARY_MONO = Dxgi::DXGI_PRESENT_STEREO_TEMPORARY_MONO;
        const DXGI_PRESENT_USE_DURATION = Dxgi::DXGI_PRESENT_USE_DURATION;
        const DXGI_PRESENT_ALLOW_TEARING = Dxgi::DXGI_PRESENT_ALLOW_TEARING;
    }
}

pub trait ISwapChain {
    fn get_buffer(&self, id: u32) -> runtime::Result<Resource>;
    fn present(&self, interval: u32, flags: u32) -> runtime::Result<()>;
    fn present_flags(&self, interval: u32, flags: SwapChainPresentFlags) -> runtime::Result<()>;
}

impl ISwapChain for SwapChain {
    fn get_buffer(&self, id: u32) -> runtime::Result<Resource> {
        unsafe { self.GetBuffer::<Resource>(id) }
    }

    //TODO: replace by present_flags
    fn present(&self, interval: u32, flags: u32) -> runtime::Result<()> {
        unsafe { self.Present(interval, flags) }
    }

    fn present_flags(&self, interval: u32, flags: SwapChainPresentFlags) -> runtime::Result<()> {
        unsafe { self.Present(interval, flags.bits()) }
    }
}

pub trait ISwapChain1 {
    fn as_swapchain0(&self) -> SwapChain;
}

impl ISwapChain1 for SwapChain1 {
    fn as_swapchain0(&self) -> SwapChain {
        self.cast::<SwapChain>().unwrap()
    }
}

pub trait ISwapChain3 {
    fn as_swapchain0(&self) -> SwapChain;
    fn get_current_back_buffer_index(&self) -> u32;
}

impl ISwapChain3 for SwapChain3 {
    fn as_swapchain0(&self) -> SwapChain {
        self.cast::<SwapChain>().unwrap()
    }

    fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.GetCurrentBackBufferIndex() }
    }
}
