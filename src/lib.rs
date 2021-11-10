#[macro_use]
extern crate bitflags;

use std::ffi::CStr;

use windows::Win32::Foundation;
use windows::Win32::Graphics::Dxgi;
use windows::{Win32::Graphics::Direct3D11, Win32::Graphics::Direct3D12};

mod com;
mod command_allocator;
mod command_list;
mod debug;
mod descriptor;
mod device;
mod dxgi;
mod heap;
mod pso;
mod query;
mod queue;
mod resource;
mod sync;

pub use crate::com::*;
pub use crate::command_allocator::*;
pub use crate::command_list::*;
pub use crate::debug::*;
pub use crate::descriptor::*;
pub use crate::device::*;
pub use crate::dxgi::*;
pub use crate::heap::*;
pub use crate::pso::*;
pub use crate::query::*;
pub use crate::queue::*;
pub use crate::resource::*;
pub use crate::sync::*;

pub type GpuAddress = Direct3D12::D3D12_GPU_VIRTUAL_ADDRESS_RANGE;
pub type Format = Dxgi::DXGI_FORMAT;
pub type Rect = Foundation::RECT;
pub type NodeMask = u32;

/// Index into the root signature.
pub type RootIndex = u32;
/// Draw vertex count.
pub type VertexCount = u32;
/// Draw vertex base offset.
pub type VertexOffset = i32;
/// Draw number of indices.
pub type IndexCount = u32;
/// Draw number of instances.
pub type InstanceCount = u32;
/// Number of work groups.
pub type WorkGroupCount = [u32; 3];

pub type TextureAddressMode = [Direct3D12::D3D12_TEXTURE_ADDRESS_MODE; 3];

pub struct SampleDesc {
    pub count: u32,
    pub quality: u32,
}

#[repr(u32)]
pub enum FeatureLevel {
    L9_1 = Direct3D11::D3D_FEATURE_LEVEL_9_1.0 as u32,
    L9_2 = Direct3D11::D3D_FEATURE_LEVEL_9_2.0 as u32,
    L9_3 = Direct3D11::D3D_FEATURE_LEVEL_9_3.0 as u32,
    L10_0 = Direct3D11::D3D_FEATURE_LEVEL_10_0.0 as u32,
    L10_1 = Direct3D11::D3D_FEATURE_LEVEL_10_1.0 as u32,
    L11_0 = Direct3D11::D3D_FEATURE_LEVEL_11_0.0 as u32,
    L11_1 = Direct3D11::D3D_FEATURE_LEVEL_11_1.0 as u32,
    L12_0 = Direct3D11::D3D_FEATURE_LEVEL_12_0.0 as u32,
    L12_1 = Direct3D11::D3D_FEATURE_LEVEL_12_1.0 as u32,
}

pub type Blob = WeakPtr<Direct3D11::ID3DBlob>;

pub type Error = WeakPtr<Direct3D11::ID3DBlob>;
impl Error {
    pub unsafe fn as_c_str(&self) -> &CStr {
        debug_assert!(!self.is_null());
        let data = self.GetBufferPointer();
        CStr::from_ptr(data as *const _ as *const _)
    }
}

#[cfg(feature = "libloading")]
#[derive(Debug)]
pub struct D3D12Lib {
    lib: libloading::Library,
}

#[cfg(feature = "libloading")]
impl D3D12Lib {
    pub fn new() -> Result<Self, libloading::Error> {
        unsafe { libloading::Library::new("Direct3D12.dll").map(|lib| D3D12Lib { lib }) }
    }
}
