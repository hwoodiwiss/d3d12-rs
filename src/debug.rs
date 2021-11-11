use crate::com::WeakPtr;
use windows::runtime::{self, Interface};
use windows::Win32::Graphics::Direct3D12;

pub type Debug = WeakPtr<Direct3D12::ID3D12Debug>;

#[cfg(feature = "libloading")]
impl crate::D3D12Lib {
    pub fn get_debug_interface(&self) -> Result<runtime::Result<Debug>, libloading::Error> {
        type Fun = extern "system" fn(&runtime::GUID, *mut *mut std::ffi::c_void) -> u32;

        let mut debug = Debug::null();
        let hr = unsafe {
            let func: libloading::Symbol<Fun> = self.lib.get(b"Direct3D12GetDebugInterface")?;
            func(&Direct3D12::ID3D12Debug::IID, debug.mut_void())
        };

        let hr = runtime::HRESULT(hr);
        if hr.is_ok() {
            Ok(Ok(debug))
        } else {
            Ok(runtime::Result::Err(runtime::Error::new(
                hr,
                hr.message().as_str(),
            )))
        }
    }
}

impl Debug {
    #[cfg(feature = "implicit-link")]
    pub fn get_interface() -> runtime::Result<Self> {
        let mut debug: Option<Direct3D12::ID3D12Debug> = None;
        let hr = unsafe { Direct3D12::D3D12GetDebugInterface(&mut debug) };

        hr.map(|()| unsafe { WeakPtr::from_raw(&mut debug.unwrap()) })
    }

    pub fn enable_layer(&self) {
        unsafe { self.EnableDebugLayer() }
    }
}
