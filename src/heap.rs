use crate::com::WeakPtr;
use windows::Win32::Graphics::Direct3D12;

pub type Heap = WeakPtr<Direct3D12::ID3D12Heap>;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum HeapType {
    Default = Direct3D12::D3D12_HEAP_TYPE_DEFAULT.0 as u32,
    Upload = Direct3D12::D3D12_HEAP_TYPE_UPLOAD.0 as u32,
    Readback = Direct3D12::D3D12_HEAP_TYPE_READBACK.0 as u32,
    Custom = Direct3D12::D3D12_HEAP_TYPE_CUSTOM.0 as u32,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum CpuPageProperty {
    Unknown = Direct3D12::D3D12_CPU_PAGE_PROPERTY_UNKNOWN.0 as u32,
    NotAvailable = Direct3D12::D3D12_CPU_PAGE_PROPERTY_NOT_AVAILABLE.0 as u32,
    WriteCombine = Direct3D12::D3D12_CPU_PAGE_PROPERTY_WRITE_COMBINE.0 as u32,
    WriteBack = Direct3D12::D3D12_CPU_PAGE_PROPERTY_WRITE_BACK.0 as u32,
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum MemoryPool {
    Unknown = Direct3D12::D3D12_MEMORY_POOL_UNKNOWN.0 as u32,
    L0 = Direct3D12::D3D12_MEMORY_POOL_L0.0 as u32,
    L1 = Direct3D12::D3D12_MEMORY_POOL_L1.0 as u32,
}

bitflags! {
    pub struct HeapFlags: u32 {
        const NONE = Direct3D12::D3D12_HEAP_FLAG_NONE.0 as u32;
        const SHARED = Direct3D12::D3D12_HEAP_FLAG_SHARED.0 as u32;
        const DENY_BUFFERS = Direct3D12::D3D12_HEAP_FLAG_DENY_BUFFERS.0 as u32;
        const ALLOW_DISPLAY = Direct3D12::D3D12_HEAP_FLAG_ALLOW_DISPLAY.0 as u32;
        const SHARED_CROSS_ADAPTER = Direct3D12::D3D12_HEAP_FLAG_SHARED_CROSS_ADAPTER.0 as u32;
        const DENT_RT_DS_TEXTURES = Direct3D12::D3D12_HEAP_FLAG_DENY_RT_DS_TEXTURES.0 as u32;
        const DENY_NON_RT_DS_TEXTURES = Direct3D12::D3D12_HEAP_FLAG_DENY_NON_RT_DS_TEXTURES.0 as u32;
        const HARDWARE_PROTECTED = Direct3D12::D3D12_HEAP_FLAG_HARDWARE_PROTECTED.0 as u32;
        const ALLOW_WRITE_WATCH = Direct3D12::D3D12_HEAP_FLAG_ALLOW_WRITE_WATCH.0 as u32;
        const ALLOW_ALL_BUFFERS_AND_TEXTURES = Direct3D12::D3D12_HEAP_FLAG_ALLOW_ALL_BUFFERS_AND_TEXTURES.0 as u32;
        const ALLOW_ONLY_BUFFERS = Direct3D12::D3D12_HEAP_FLAG_ALLOW_ONLY_BUFFERS.0 as u32;
        const ALLOW_ONLY_NON_RT_DS_TEXTURES = Direct3D12::D3D12_HEAP_FLAG_ALLOW_ONLY_NON_RT_DS_TEXTURES.0 as u32;
        const ALLOW_ONLY_RT_DS_TEXTURES = Direct3D12::D3D12_HEAP_FLAG_ALLOW_ONLY_RT_DS_TEXTURES.0 as u32;
    }
}

#[repr(transparent)]
pub struct HeapProperties(pub Direct3D12::D3D12_HEAP_PROPERTIES);
impl HeapProperties {
    pub fn new(
        heap_type: HeapType,
        cpu_page_property: CpuPageProperty,
        memory_pool_preference: MemoryPool,
        creation_node_mask: u32,
        visible_node_mask: u32,
    ) -> Self {
        HeapProperties(Direct3D12::D3D12_HEAP_PROPERTIES {
            Type: Direct3D12::D3D12_HEAP_TYPE(heap_type as _),
            CPUPageProperty: Direct3D12::D3D12_CPU_PAGE_PROPERTY(cpu_page_property as _),
            MemoryPoolPreference: Direct3D12::D3D12_MEMORY_POOL(memory_pool_preference as _),
            CreationNodeMask: creation_node_mask,
            VisibleNodeMask: visible_node_mask,
        })
    }
}

#[repr(transparent)]
pub struct HeapDesc(Direct3D12::D3D12_HEAP_DESC);
impl HeapDesc {
    pub fn new(
        size_in_bytes: u64,
        properties: HeapProperties,
        alignment: u64,
        flags: HeapFlags,
    ) -> Self {
        HeapDesc(Direct3D12::D3D12_HEAP_DESC {
            SizeInBytes: size_in_bytes,
            Properties: properties.0,
            Alignment: alignment,
            Flags: Direct3D12::D3D12_HEAP_FLAGS(flags.bits()),
        })
    }
}
