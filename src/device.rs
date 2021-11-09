//! Device

use crate::{
    com::WeakPtr,
    command_list::{CmdListType, CommandSignature, IndirectArgument},
    descriptor::{CpuDescriptor, DescriptorHeapFlags, DescriptorHeapType, RenderTargetViewDesc},
    heap::{Heap, HeapFlags, HeapProperties},
    pso, query, queue, Blob, CachedPSO, CommandAllocator, CommandQueue, D3DResult, DescriptorHeap,
    Fence, GraphicsCommandList, NodeMask, PipelineState, QueryHeap, Resource, RootSignature,
    Shader, TextureAddressMode,
};
use std::ops::Range;
use windows::{
    runtime::{self, IUnknown, Interface},
    Win32::Graphics::{Direct3D11, Direct3D12},
};

pub type Device = WeakPtr<Direct3D12::ID3D12Device>;

#[cfg(feature = "libloading")]
impl crate::Direct3D12Lib {
    pub fn create_device<I: Interface>(
        &self,
        adapter: WeakPtr<I>,
        feature_level: crate::FeatureLevel,
    ) -> Result<D3DResult<Device>, libloading::Error> {
        type Fun = extern "system" fn(
            *mut IUnknown,
            Direct3D11::D3D_FEATURE_LEVEL,
            &runtime::GUID,
            *mut *mut std::ffi::c_void,
        ) -> crate::HRESULT;

        let mut device = Device::null();
        let hr = unsafe {
            let func: libloading::Symbol<Fun> = self.lib.get(b"Direct3D12CreateDevice")?;
            func(
                adapter.as_unknown() as *const _ as *mut _,
                Direct3D11::D3D_FEATURE_LEVEL(feature_level as _),
                &Direct3D12::ID3D12Device::IID,
                device.mut_void(),
            )
        };

        Ok((device, hr))
    }
}

impl Device {
    #[cfg(feature = "implicit-link")]
    pub fn create<I: Interface>(
        adapter: WeakPtr<I>,
        feature_level: crate::FeatureLevel,
    ) -> D3DResult<Self> {
        let mut device: Option<Direct3D12::ID3D12Device>;
        let hr = unsafe {
            Direct3D12::D3D12CreateDevice(
                adapter.as_unknown(),
                Direct3D11::D3D_FEATURE_LEVEL(feature_level as _),
                &mut device,
            )
        };

        if let Some(device) = device {
            (unsafe { WeakPtr::from_raw(&mut device) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn create_heap(
        &self,
        size_in_bytes: u64,
        properties: HeapProperties,
        alignment: u64,
        flags: HeapFlags,
    ) -> D3DResult<Heap> {
        let mut heap = Heap::null();

        let desc = Direct3D12::D3D12_HEAP_DESC {
            SizeInBytes: size_in_bytes,
            Properties: properties.0,
            Alignment: alignment,
            Flags: Direct3D12::D3D12_HEAP_FLAGS(flags.bits()),
        };

        let heap: Option<Direct3D12::ID3D12Heap>;
        let hr = unsafe { self.CreateHeap(&desc, &mut heap) };

        if let Some(heap) = heap {
            (unsafe { WeakPtr::from_raw(&mut heap) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn create_command_allocator(&self, list_type: CmdListType) -> D3DResult<CommandAllocator> {
        let mut allocator = CommandAllocator::null();
        let hr = unsafe {
            self.CreateCommandAllocator::<Direct3D12::ID3D12CommandAllocator>(
                Direct3D12::D3D12_COMMAND_LIST_TYPE(list_type as _),
            )
        };

        if let Ok(allocator) = hr {
            (unsafe { WeakPtr::from_raw(&mut allocator) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn create_command_queue(
        &self,
        list_type: CmdListType,
        priority: queue::Priority,
        flags: queue::CommandQueueFlags,
        node_mask: NodeMask,
    ) -> D3DResult<CommandQueue> {
        let desc = Direct3D12::D3D12_COMMAND_QUEUE_DESC {
            Type: Direct3D12::D3D12_COMMAND_LIST_TYPE(list_type as _),
            Priority: priority as _,
            Flags: Direct3D12::D3D12_COMMAND_QUEUE_FLAGS(flags.bits()),
            NodeMask: node_mask,
        };

        let hr = unsafe { self.CreateCommandQueue::<Direct3D12::ID3D12CommandQueue>(&desc) };

        if let Ok(queue) = hr {
            (unsafe { WeakPtr::from_raw(&mut queue) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn create_descriptor_heap(
        &self,
        num_descriptors: u32,
        heap_type: DescriptorHeapType,
        flags: DescriptorHeapFlags,
        node_mask: NodeMask,
    ) -> D3DResult<DescriptorHeap> {
        let desc = Direct3D12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE(heap_type as _),
            NumDescriptors: num_descriptors,
            Flags: Direct3D12::D3D12_DESCRIPTOR_HEAP_FLAGS(flags.bits()),
            NodeMask: node_mask,
        };

        let hr = unsafe { self.CreateDescriptorHeap::<Direct3D12::ID3D12DescriptorHeap>(&desc) };

        if let Ok(heap) = hr {
            (unsafe { WeakPtr::from_raw(&mut heap) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn get_descriptor_increment_size(&self, heap_type: DescriptorHeapType) -> u32 {
        unsafe {
            self.GetDescriptorHandleIncrementSize(Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE(
                heap_type as _,
            ))
        }
    }

    pub fn create_graphics_command_list(
        &self,
        list_type: CmdListType,
        allocator: CommandAllocator,
        initial: PipelineState,
        node_mask: NodeMask,
    ) -> D3DResult<GraphicsCommandList> {
        let hr = unsafe {
            self.CreateCommandList::<_, _, Direct3D12::ID3D12GraphicsCommandList>(
                node_mask,
                Direct3D12::D3D12_COMMAND_LIST_TYPE(list_type as _),
                allocator,
                initial,
            )
        };

        if let Ok(command_list) = hr {
            (unsafe { WeakPtr::from_raw(&mut command_list) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn create_query_heap(
        &self,
        heap_ty: query::QueryHeapType,
        count: u32,
        node_mask: NodeMask,
    ) -> D3DResult<QueryHeap> {
        let desc = Direct3D12::D3D12_QUERY_HEAP_DESC {
            Type: Direct3D12::D3D12_QUERY_HEAP_TYPE(heap_ty as _),
            Count: count,
            NodeMask: node_mask,
        };

        let mut query_heap: Option<Direct3D12::ID3D12QueryHeap>;
        let hr = unsafe { self.CreateQueryHeap(&desc, &mut query_heap) };

        if let Some(query_heap) = query_heap {
            (unsafe { WeakPtr::from_raw(&mut query_heap) }, Ok(()))
        } else {
            (WeakPtr::null(), Err(hr.err().unwrap()))
        }
    }

    pub fn create_graphics_pipeline_state(
        &self,
        _root_signature: RootSignature,
        _vs: Shader,
        _ps: Shader,
        _gs: Shader,
        _hs: Shader,
        _ds: Shader,
        _node_mask: NodeMask,
        _cached_pso: CachedPSO,
        _flags: pso::PipelineStateFlags,
    ) -> D3DResult<PipelineState> {
        unimplemented!()
    }

    pub fn create_compute_pipeline_state(
        &self,
        root_signature: RootSignature,
        cs: Shader,
        node_mask: NodeMask,
        cached_pso: CachedPSO,
        flags: pso::PipelineStateFlags,
    ) -> D3DResult<PipelineState> {
        let mut pipeline = PipelineState::null();
        let desc = Direct3D12::D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: Some(root_signature),
            CS: *cs,
            NodeMask: node_mask,
            CachedPSO: *cached_pso,
            Flags: flags.bits(),
        };

        let hr = unsafe {
            self.CreateComputePipelineState(
                &desc,
                &Direct3D12::ID3D12PipelineState::uuidof(),
                pipeline.mut_void(),
            )
        };

        (pipeline, hr)
    }

    pub fn create_sampler(
        &self,
        sampler: CpuDescriptor,
        filter: Direct3D12::D3D12_FILTER,
        address_mode: TextureAddressMode,
        mip_lod_bias: f32,
        max_anisotropy: u32,
        comparison_op: Direct3D12::D3D12_COMPARISON_FUNC,
        border_color: [f32; 4],
        lod: Range<f32>,
    ) {
        let desc = Direct3D12::D3D12_SAMPLER_DESC {
            Filter: filter,
            AddressU: address_mode[0],
            AddressV: address_mode[1],
            AddressW: address_mode[2],
            MipLODBias: mip_lod_bias,
            MaxAnisotropy: max_anisotropy,
            ComparisonFunc: comparison_op,
            BorderColor: border_color,
            MinLOD: lod.start,
            MaxLOD: lod.end,
        };

        unsafe {
            self.CreateSampler(&desc, sampler);
        }
    }

    pub fn create_root_signature(
        &self,
        blob: Blob,
        node_mask: NodeMask,
    ) -> D3DResult<RootSignature> {
        let mut signature = RootSignature::null();
        let hr = unsafe {
            self.CreateRootSignature(
                node_mask,
                blob.GetBufferPointer(),
                blob.GetBufferSize(),
                &Direct3D12::ID3D12RootSignature::uuidof(),
                signature.mut_void(),
            )
        };

        (signature, hr)
    }

    pub fn create_command_signature(
        &self,
        root_signature: RootSignature,
        arguments: &[IndirectArgument],
        stride: u32,
        node_mask: NodeMask,
    ) -> D3DResult<CommandSignature> {
        let mut signature = CommandSignature::null();
        let desc = Direct3D12::D3D12_COMMAND_SIGNATURE_DESC {
            ByteStride: stride,
            NumArgumentDescs: arguments.len() as _,
            pArgumentDescs: arguments.as_ptr() as *const _,
            NodeMask: node_mask,
        };

        let hr = unsafe {
            self.CreateCommandSignature(
                &desc,
                root_signature.as_mut_ptr(),
                &Direct3D12::ID3D12CommandSignature::uuidof(),
                signature.mut_void(),
            )
        };

        (signature, hr)
    }

    pub fn create_render_target_view(
        &self,
        resource: Resource,
        desc: &RenderTargetViewDesc,
        descriptor: CpuDescriptor,
    ) {
        unsafe {
            self.CreateRenderTargetView(resource.as_mut_ptr(), &desc.0 as *const _, descriptor);
        }
    }

    // TODO: interface not complete
    pub fn create_fence(&self, initial: u64) -> D3DResult<Fence> {
        let mut fence = Fence::null();
        let hr = unsafe {
            self.CreateFence(
                initial,
                Direct3D12::D3D12_FENCE_FLAG_NONE,
                &Direct3D12::ID3D12Fence::uuidof(),
                fence.mut_void(),
            )
        };

        (fence, hr)
    }
}
