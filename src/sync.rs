use std::ptr;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Direct3D12;
use windows::Win32::System::Threading::{CreateEventA, WaitForSingleObject};
use windows::{self};

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Event(pub HANDLE);
impl Event {
    pub fn create(manual_reset: bool, initial_state: bool) -> Self {
        Event(unsafe { CreateEventA(ptr::null_mut(), manual_reset, initial_state, None) })
    }

    // TODO: return value
    pub fn wait(&self, timeout_ms: u32) -> u32 {
        unsafe { WaitForSingleObject(self.0, timeout_ms) }
    }
}

pub type Fence = Direct3D12::ID3D12Fence;

pub trait IFence {
    fn set_event_on_completion(
        &self,
        event: Event,
        value: u64,
    ) -> Result<(), windows::runtime::Error>;
    fn get_value(&self) -> u64;
    fn signal(&self, value: u64) -> Result<(), windows::runtime::Error>;
}

impl IFence for Fence {
    fn set_event_on_completion(
        &self,
        event: Event,
        value: u64,
    ) -> Result<(), windows::runtime::Error> {
        unsafe { self.SetEventOnCompletion(value, event.0) }
    }

    fn get_value(&self) -> u64 {
        unsafe { self.GetCompletedValue() }
    }

    fn signal(&self, value: u64) -> Result<(), windows::runtime::Error> {
        unsafe { self.Signal(value) }
    }
}
