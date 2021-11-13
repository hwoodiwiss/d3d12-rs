//! Command Allocator

use windows::Win32::Graphics::Direct3D12;

pub type CommandAllocator = Direct3D12::ID3D12CommandAllocator;
