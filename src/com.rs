use std::{
    ffi::c_void,
    fmt,
    hash::{Hash, Hasher},
    mem,
    ops::Deref,
    ptr,
};
use windows::runtime::{self, IUnknown, Interface, IntoParam, Param, GUID};

#[repr(transparent)]
pub struct WeakPtr<T>(*mut T);

impl<T> WeakPtr<T> {
    pub fn null() -> Self {
        WeakPtr(ptr::null_mut())
    }

    pub unsafe fn from_raw(raw: *mut T) -> Self {
        WeakPtr(raw)
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.0
    }

    pub unsafe fn mut_void(&mut self) -> *mut *mut c_void {
        &mut self.0 as *mut *mut _ as *mut *mut _
    }
}

impl<T: Interface> WeakPtr<T> {
    pub unsafe fn as_unknown(&self) -> &IUnknown {
        debug_assert!(!self.is_null());
        println!("{:?}", self.0);
        let uk = mem::transmute_copy(self);
        println!("{:?}", uk);

        uk
    }

    // Cast creates a new WeakPtr requiring explicit destroy call.
    pub unsafe fn cast<U>(&self) -> Result<WeakPtr<U>, runtime::Error>
    where
        U: Interface,
    {
        (*self.0).cast::<U>().map(|mut u| WeakPtr::from_raw(&mut u))
    }

    // Destroying one instance of the WeakPtr will invalidate all
    // copies and clones.
    pub unsafe fn destroy(&self) {
        mem::drop(self.as_unknown());
    }
}

impl<'a, T: Interface> IntoParam<'a, T> for WeakPtr<T> {
    fn into_param(self) -> Param<'a, T> {
        unsafe { Param::Borrowed(self.0.as_ref().unwrap()) }
    }
}

impl<T> Clone for WeakPtr<T> {
    fn clone(&self) -> Self {
        WeakPtr(self.0)
    }
}

impl<T> Copy for WeakPtr<T> {}

impl<T> Deref for WeakPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        debug_assert!(!self.is_null());
        unsafe { &*self.0 }
    }
}

impl<T> fmt::Debug for WeakPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WeakPtr( ptr: {:?} )", self.0)
    }
}

impl<T> PartialEq<*mut T> for WeakPtr<T> {
    fn eq(&self, other: &*mut T) -> bool {
        self.0 == *other
    }
}

impl<T> PartialEq for WeakPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Hash for WeakPtr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

unsafe impl<T: Interface> Interface for WeakPtr<T> {
    type Vtable = T::Vtable;

    const IID: GUID = T::IID;
}
