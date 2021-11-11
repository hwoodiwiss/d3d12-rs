use crate::{com::WeakPtr, CommandQueue, Resource, SampleDesc};
use std::{ffi::c_void, ptr};
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

pub type Adapter1 = WeakPtr<Dxgi::IDXGIAdapter1>;
pub type Factory2 = WeakPtr<Dxgi::IDXGIFactory2>;
pub type Factory4 = WeakPtr<Dxgi::IDXGIFactory4>;
pub type InfoQueue = WeakPtr<Dxgi::IDXGIInfoQueue>;
pub type SwapChain = WeakPtr<Dxgi::IDXGISwapChain>;
pub type SwapChain1 = WeakPtr<Dxgi::IDXGISwapChain1>;
pub type SwapChain3 = WeakPtr<Dxgi::IDXGISwapChain3>;

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
        type Fun = extern "system" fn(u32, *const runtime::GUID, *mut *mut c_void) -> u32;

        let mut queue = InfoQueue::null();
        let hr = unsafe {
            let func: libloading::Symbol<Fun> = self.lib.get(b"DXGIGetDebugInterface1")?;
            func(0, &Dxgi::IDXGIInfoQueue::IID, queue.mut_void())
        };
        let hr = runtime::HRESULT(hr);
        if hr.is_ok() {
            Ok(Ok(queue))
        } else {
            Ok(runtime::Result::Err(runtime::Error::new(
                hr,
                hr.message().as_str(),
            )))
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

impl Factory2 {
    // TODO: interface not complete
    pub fn create_swapchain_for_hwnd(
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

        let hr = unsafe {
            let mut device: Option<IDXGIDevice2> = None;
            queue.GetDevice(&mut device)?;
            let device = device.unwrap();
            self.CreateSwapChainForHwnd(device, hwnd, &desc, ptr::null(), None)
        };

        hr.map(|mut sc| unsafe { WeakPtr::from_raw(&mut sc) })
    }
}

impl Factory4 {
    #[cfg(feature = "implicit-link")]
    pub fn create(flags: FactoryCreationFlags) -> runtime::Result<Self> {
        unsafe { Dxgi::CreateDXGIFactory2::<Self>(flags.bits()) }
    }

    pub fn as_factory2(&self) -> Factory2 {
        unsafe { Factory2::from_raw(self.as_mut_ptr() as *mut _) }
    }

    pub fn enumerate_adapters(&self, id: u32) -> runtime::Result<Adapter1> {
        let hr = unsafe { self.EnumAdapters1(id) };

        hr.map(|mut adapter| unsafe { WeakPtr::from_raw(&mut adapter) })
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

impl SwapChain {
    pub fn get_buffer(&self, id: u32) -> runtime::Result<Resource> {
        unsafe { self.GetBuffer::<Resource>(id) }
    }

    //TODO: replace by present_flags
    pub fn present(&self, interval: u32, flags: u32) -> runtime::Result<()> {
        unsafe { self.Present(interval, flags) }
    }

    pub fn present_flags(
        &self,
        interval: u32,
        flags: SwapChainPresentFlags,
    ) -> runtime::Result<()> {
        unsafe { self.Present(interval, flags.bits()) }
    }
}

impl SwapChain1 {
    pub fn as_swapchain0(&self) -> SwapChain {
        unsafe { SwapChain::from_raw(self.as_mut_ptr() as *mut _) }
    }
}

impl SwapChain3 {
    pub fn as_swapchain0(&self) -> SwapChain {
        unsafe { SwapChain::from_raw(self.as_mut_ptr() as *mut _) }
    }

    pub fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.GetCurrentBackBufferIndex() }
    }
}
