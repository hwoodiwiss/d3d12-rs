use crate::{sync::Fence, CommandList};
use windows::{
    runtime::{self, Interface},
    Win32::Graphics::Direct3D12::{self, ID3D12CommandList},
};

#[repr(u32)]
pub enum Priority {
    Normal = Direct3D12::D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0 as u32,
    High = Direct3D12::D3D12_COMMAND_QUEUE_PRIORITY_HIGH.0 as u32,
    GlobalRealtime = Direct3D12::D3D12_COMMAND_QUEUE_PRIORITY_GLOBAL_REALTIME.0 as u32,
}

bitflags! {
    pub struct CommandQueueFlags: u32 {
        const DISABLE_GPU_TIMEOUT = Direct3D12::D3D12_COMMAND_QUEUE_FLAG_DISABLE_GPU_TIMEOUT.0 as u32;
    }
}

pub trait ICommandQueue {
    fn execute_command_lists(&self, command_lists: &[CommandList]);
    fn signal(&self, fence: Fence, value: u64) -> Result<(), runtime::Error>;
}

pub type CommandQueue = Direct3D12::ID3D12CommandQueue;

impl ICommandQueue for CommandQueue {
    fn execute_command_lists(&self, command_lists: &[CommandList]) {
        let command_lists = command_lists
            .iter()
            .map(|c| Some(c.0.cast::<ID3D12CommandList>().expect("Tried to execute command list lists containing objects other than command lists")))
            .collect::<Box<[_]>>();
        unsafe { self.ExecuteCommandLists(command_lists.len() as _, command_lists.as_ptr()) }
    }

    fn signal(&self, fence: Fence, value: u64) -> Result<(), runtime::Error> {
        unsafe { self.Signal(fence, value) }
    }
}
