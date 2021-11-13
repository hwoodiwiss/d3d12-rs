use windows::runtime::{self};
use windows::Win32::Graphics::Direct3D12;

pub type Debug = Direct3D12::ID3D12Debug;

#[cfg(feature = "libloading")]
impl crate::D3D12Lib {
    pub fn get_debug_interface(&self) -> Result<runtime::Result<Debug>, libloading::Error> {
        let mut debug = None;

        let hr = unsafe { Direct3D12::D3D12GetDebugInterface::<Debug>(&mut debug) };

        match hr {
            Ok(_) => Ok(Ok(debug.unwrap())),
            Err(err) => Ok(runtime::Result::Err(runtime::Error::new(
                err.code(),
                err.message().as_str(),
            ))),
        }
    }
}

pub trait IDebug {
    #[cfg(feature = "implicit-link")]
    fn get_interface() -> runtime::Result<Debug>;
    fn enable_layer(&self);
}

impl IDebug for Debug {
    #[cfg(feature = "implicit-link")]
    fn get_interface() -> runtime::Result<Self> {
        let mut debug: Option<Direct3D12::ID3D12Debug> = None;
        let hr = unsafe { Direct3D12::D3D12GetDebugInterface(&mut debug) };

        hr.map(|()| debug.unwrap())
    }

    fn enable_layer(&self) {
        unsafe { self.EnableDebugLayer() }
    }
}
