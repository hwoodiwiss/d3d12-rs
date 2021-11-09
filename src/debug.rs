use crate::com::WeakPtr;
use windows::runtime::{self, Interface};
#[cfg(any(feature = "libloading", feature = "implicit-link"))]
use windows::Win32::Graphics::Direct3D12;

pub type Debug = WeakPtr<Direct3D12::ID3D12Debug>;

#[cfg(feature = "libloading")]
impl crate::Direct3D12Lib {
    pub fn get_debug_interface(&self) -> Result<crate::D3DResult<Debug>, libloading::Error> {
        type Fun =
            extern "system" fn(&runtime::GUID, *mut *mut std::ffi::c_void) -> runtime::Result<()>;

        let mut debug = Debug::null();
        let hr = unsafe {
            let func: libloading::Symbol<Fun> = self.lib.get(b"Direct3D12GetDebugInterface")?;
            func(&Direct3D12::ID3D12Debug::IID, debug.mut_void())
        };

        Ok((debug, hr))
    }
}

impl Debug {
    #[cfg(feature = "implicit-link")]
    pub fn get_interface() -> crate::D3DResult<Self> {
        let mut debug = Debug::null();
        unsafe {
            let mut debug_opt = &mut Some(*debug.as_ptr());
            let hr = Direct3D12::D3D12GetDebugInterface(debug_opt);

            (debug, hr)
        }
    }

    pub fn enable_layer(&self) {
        unsafe { self.EnableDebugLayer() }
    }
}
