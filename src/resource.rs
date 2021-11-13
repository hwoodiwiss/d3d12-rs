//! GPU Resource

use crate::Rect;
use core::ffi;
use std::{ops::Range, ptr};
use windows::{runtime, Win32::Graphics::Direct3D12};

pub type Subresource = u32;

pub struct DiscardRegion<'a> {
    pub rects: &'a mut [Rect],
    pub subregions: Range<Subresource>,
}

pub type Resource = Direct3D12::ID3D12Resource;

pub trait IResource {
    fn map(
        &self,
        subresource: Subresource,
        read_range: Option<Range<usize>>,
    ) -> runtime::Result<*mut ffi::c_void>;
    fn unmap(&self, subresource: Subresource, write_range: Option<Range<usize>>);
    fn gpu_virtual_address(&self) -> u64;
}

impl IResource for Resource {
    ///
    fn map(
        &self,
        subresource: Subresource,
        read_range: Option<Range<usize>>,
    ) -> runtime::Result<*mut ffi::c_void> {
        let mut ptr = ptr::null_mut();
        let read_range = read_range.map(|r| Direct3D12::D3D12_RANGE {
            Begin: r.start,
            End: r.end,
        });
        let read = match read_range {
            Some(ref r) => r as *const _,
            None => ptr::null(),
        };
        let hr = unsafe { self.Map(subresource, read, &mut ptr) };

        hr.map(|()| ptr)
    }

    fn unmap(&self, subresource: Subresource, write_range: Option<Range<usize>>) {
        let write_range = write_range.map(|r| Direct3D12::D3D12_RANGE {
            Begin: r.start,
            End: r.end,
        });
        let write = match write_range {
            Some(ref r) => r as *const _,
            None => ptr::null(),
        };

        unsafe { self.Unmap(subresource, write) };
    }

    fn gpu_virtual_address(&self) -> u64 {
        unsafe { self.GetGPUVirtualAddress() }
    }
}
