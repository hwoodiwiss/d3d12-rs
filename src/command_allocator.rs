//! Command Allocator

use windows::Win32::Graphics::Direct3D12;

use crate::WeakPtr;

pub type CommandAllocator = WeakPtr<Direct3D12::ID3D12CommandAllocator>;

impl CommandAllocator {
    pub fn reset(&self) {
        unsafe {
            self.Reset();
        }
    }
}
