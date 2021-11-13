//! Graphics command list

use crate::{
    resource::DiscardRegion, CommandAllocator, CpuDescriptor, DescriptorHeap, Format, GpuAddress,
    GpuDescriptor, IndexCount, InstanceCount, PipelineState, Rect, Resource, RootIndex,
    RootSignature, Subresource, VertexCount, VertexOffset, WorkGroupCount,
};
use std::{mem, ptr};
use windows::{
    runtime::{self, Interface},
    Win32::Graphics::Direct3D12::{self},
};

type IndirectArgumentDescVertexBuffer = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC_0_4;
type IndirectArgumentDescConstant = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC_0_1;
type IndirectArgumentDescConstantBufferView = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC_0_0;
type IndirectArgumentDescShaderResourceView = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC_0_2;
type IndirectArgumentDescUnorderedAccessView = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC_0_3;

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum CmdListType {
    Direct = Direct3D12::D3D12_COMMAND_LIST_TYPE_DIRECT.0 as u32,
    Bundle = Direct3D12::D3D12_COMMAND_LIST_TYPE_BUNDLE.0 as u32,
    Compute = Direct3D12::D3D12_COMMAND_LIST_TYPE_COMPUTE.0 as u32,
    Copy = Direct3D12::D3D12_COMMAND_LIST_TYPE_COPY.0 as u32,
    // VideoDecode = Direct3D12::D3D12_COMMAND_LIST_TYPE_VIDEO_DECODE,
    // VideoProcess = Direct3D12::D3D12_COMMAND_LIST_TYPE_VIDEO_PROCESS,
}

bitflags! {
    pub struct ClearFlags: u32 {
        const DEPTH = Direct3D12::D3D12_CLEAR_FLAG_DEPTH.0 as u32;
        const STENCIL = Direct3D12::D3D12_CLEAR_FLAG_STENCIL.0 as u32;
    }
}

#[repr(transparent)]
pub struct IndirectArgument(Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC);

impl IndirectArgument {
    pub fn draw() -> Self {
        IndirectArgument(Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_DRAW,
            ..unsafe { mem::zeroed() }
        })
    }

    pub fn draw_indexed() -> Self {
        IndirectArgument(Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_DRAW_INDEXED,
            ..unsafe { mem::zeroed() }
        })
    }

    pub fn dispatch() -> Self {
        IndirectArgument(Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_DISPATCH,
            ..unsafe { mem::zeroed() }
        })
    }

    pub fn vertex_buffer(slot: u32) -> Self {
        let mut desc = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_VERTEX_BUFFER_VIEW,
            ..unsafe { mem::zeroed() }
        };
        desc.Anonymous.VertexBuffer = IndirectArgumentDescVertexBuffer { Slot: slot };
        IndirectArgument(desc)
    }

    pub fn constant(root_index: RootIndex, dest_offset_words: u32, count: u32) -> Self {
        let mut desc = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_CONSTANT,
            ..unsafe { mem::zeroed() }
        };
        desc.Anonymous.Constant = IndirectArgumentDescConstant {
            RootParameterIndex: root_index,
            DestOffsetIn32BitValues: dest_offset_words,
            Num32BitValuesToSet: count,
        };
        IndirectArgument(desc)
    }

    pub fn constant_buffer_view(root_index: RootIndex) -> Self {
        let mut desc = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_CONSTANT_BUFFER_VIEW,
            ..unsafe { mem::zeroed() }
        };
        desc.Anonymous.ConstantBufferView = IndirectArgumentDescConstantBufferView {
            RootParameterIndex: root_index,
        };
        IndirectArgument(desc)
    }

    pub fn shader_resource_view(root_index: RootIndex) -> Self {
        let mut desc = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_SHADER_RESOURCE_VIEW,
            ..unsafe { mem::zeroed() }
        };
        desc.Anonymous.ShaderResourceView = IndirectArgumentDescShaderResourceView {
            RootParameterIndex: root_index,
        };
        IndirectArgument(desc)
    }

    pub fn unordered_access_view(root_index: RootIndex) -> Self {
        let mut desc = Direct3D12::D3D12_INDIRECT_ARGUMENT_DESC {
            Type: Direct3D12::D3D12_INDIRECT_ARGUMENT_TYPE_UNORDERED_ACCESS_VIEW,
            ..unsafe { mem::zeroed() }
        };
        desc.Anonymous.UnorderedAccessView = IndirectArgumentDescUnorderedAccessView {
            RootParameterIndex: root_index,
        };
        IndirectArgument(desc)
    }
}

#[repr(transparent)]
pub struct ResourceBarrier(Direct3D12::D3D12_RESOURCE_BARRIER);

impl ResourceBarrier {
    pub fn transition(
        resource: Resource,
        subresource: Subresource,
        state_before: Direct3D12::D3D12_RESOURCE_STATES,
        state_after: Direct3D12::D3D12_RESOURCE_STATES,
        flags: Direct3D12::D3D12_RESOURCE_BARRIER_FLAGS,
    ) -> Self {
        let mut barrier = Direct3D12::D3D12_RESOURCE_BARRIER {
            Type: Direct3D12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: flags,
            ..unsafe { mem::zeroed() }
        };
        unsafe {
            *barrier.Anonymous.Transition = Direct3D12::D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: Some(resource.0.cast::<Direct3D12::ID3D12Resource>().unwrap()),
                Subresource: subresource,
                StateBefore: state_before,
                StateAfter: state_after,
            };
        }
        ResourceBarrier(barrier)
    }
}

pub type CommandSignature = Direct3D12::ID3D12CommandSignature;
pub type CommandList = Direct3D12::ID3D12CommandList;

pub trait IGraphicsCommandList {
    fn as_list(&self) -> CommandList;
    fn close(&self) -> Result<(), runtime::Error>;
    fn reset(
        &self,
        allocator: CommandAllocator,
        initial_pso: Option<PipelineState>,
    ) -> Result<(), runtime::Error>;
    fn discard_resource(&self, resource: Resource, region: DiscardRegion);
    fn clear_depth_stencil_view(
        &self,
        dsv: CpuDescriptor,
        flags: ClearFlags,
        depth: f32,
        stencil: u8,
        rects: &[Rect],
    );
    fn clear_render_target_view(&self, rtv: CpuDescriptor, color: [f32; 4], rects: &[Rect]);
    fn dispatch(&self, count: WorkGroupCount);
    fn draw(
        &self,
        num_vertices: VertexCount,
        num_instances: InstanceCount,
        start_vertex: VertexCount,
        start_instance: InstanceCount,
    );
    fn draw_indexed(
        &self,
        num_indices: IndexCount,
        num_instances: InstanceCount,
        start_index: IndexCount,
        base_vertex: VertexOffset,
        start_instance: InstanceCount,
    );
    fn set_index_buffer(&self, gpu_address: GpuAddress, size: u32, format: Format);
    fn set_blend_factor(&self, factor: [f32; 4]);
    fn set_stencil_reference(&self, reference: u32);
    fn set_pipeline_state(&self, pso: PipelineState);
    fn execute_bundle(&self, bundle: GraphicsCommandList);
    fn set_descriptor_heaps(&self, heaps: &[DescriptorHeap]);
    fn set_compute_root_signature(&self, signature: RootSignature);
    fn set_graphics_root_signature(&self, signature: RootSignature);
    fn set_compute_root_descriptor_table(
        &self,
        root_index: RootIndex,
        base_descriptor: GpuDescriptor,
    );
    fn set_compute_root_constant_buffer_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    );
    fn set_compute_root_shader_resource_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    );
    fn set_compute_root_unordered_access_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    );
    fn set_compute_root_constant(&self, root_index: RootIndex, value: u32, dest_offset_words: u32);
    fn set_graphics_root_descriptor_table(
        &self,
        root_index: RootIndex,
        base_descriptor: GpuDescriptor,
    );
    fn set_graphics_root_constant_buffer_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    );
    fn set_graphics_root_shader_resource_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    );
    fn set_graphics_root_unordered_access_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    );
    fn set_graphics_root_constant(&self, root_index: RootIndex, value: u32, dest_offset_words: u32);
    fn resource_barrier(&self, barriers: &[ResourceBarrier]);
}

pub type GraphicsCommandList = Direct3D12::ID3D12GraphicsCommandList;

impl IGraphicsCommandList for GraphicsCommandList {
    fn as_list(&self) -> CommandList {
        self.cast::<CommandList>().unwrap()
    }

    fn close(&self) -> Result<(), runtime::Error> {
        unsafe { self.Close() }
    }

    fn reset(
        &self,
        allocator: CommandAllocator,
        initial_pso: Option<PipelineState>,
    ) -> Result<(), runtime::Error> {
        unsafe { self.Reset(allocator, initial_pso) }
    }

    fn discard_resource(&self, resource: Resource, region: DiscardRegion) {
        debug_assert!(region.subregions.start < region.subregions.end);
        unsafe {
            let rects = region.rects;
            self.DiscardResource(
                resource,
                &Direct3D12::D3D12_DISCARD_REGION {
                    NumRects: rects.len() as _,
                    pRects: (*rects).as_mut_ptr(),
                    FirstSubresource: region.subregions.start,
                    NumSubresources: region.subregions.end - region.subregions.start - 1,
                },
            );
        }
    }

    fn clear_depth_stencil_view(
        &self,
        dsv: CpuDescriptor,
        flags: ClearFlags,
        depth: f32,
        stencil: u8,
        rects: &[Rect],
    ) {
        let num_rects = rects.len() as _;
        let rects = if num_rects > 0 {
            rects.as_ptr()
        } else {
            ptr::null()
        };
        unsafe {
            self.ClearDepthStencilView(
                dsv,
                Direct3D12::D3D12_CLEAR_FLAGS(flags.bits()),
                depth,
                stencil,
                num_rects,
                rects,
            );
        }
    }

    fn clear_render_target_view(&self, rtv: CpuDescriptor, color: [f32; 4], rects: &[Rect]) {
        let num_rects = rects.len() as _;
        let rects = if num_rects > 0 {
            rects.as_ptr()
        } else {
            ptr::null()
        };
        unsafe {
            self.ClearRenderTargetView(rtv, color.as_ptr(), num_rects, rects);
        }
    }

    fn dispatch(&self, count: WorkGroupCount) {
        unsafe {
            self.Dispatch(count[0], count[1], count[2]);
        }
    }

    fn draw(
        &self,
        num_vertices: VertexCount,
        num_instances: InstanceCount,
        start_vertex: VertexCount,
        start_instance: InstanceCount,
    ) {
        unsafe {
            self.DrawInstanced(num_vertices, num_instances, start_vertex, start_instance);
        }
    }

    fn draw_indexed(
        &self,
        num_indices: IndexCount,
        num_instances: InstanceCount,
        start_index: IndexCount,
        base_vertex: VertexOffset,
        start_instance: InstanceCount,
    ) {
        unsafe {
            self.DrawIndexedInstanced(
                num_indices,
                num_instances,
                start_index,
                base_vertex,
                start_instance,
            );
        }
    }

    fn set_index_buffer(&self, gpu_address: GpuAddress, size: u32, format: Format) {
        let ibv = Direct3D12::D3D12_INDEX_BUFFER_VIEW {
            BufferLocation: gpu_address,
            SizeInBytes: size,
            Format: format,
        };
        unsafe {
            self.IASetIndexBuffer(&ibv);
        }
    }

    fn set_blend_factor(&self, factor: [f32; 4]) {
        unsafe {
            self.OMSetBlendFactor(factor.as_ptr());
        }
    }

    fn set_stencil_reference(&self, reference: u32) {
        unsafe {
            self.OMSetStencilRef(reference);
        }
    }

    fn set_pipeline_state(&self, pso: PipelineState) {
        unsafe {
            self.SetPipelineState(pso);
        }
    }

    fn execute_bundle(&self, bundle: GraphicsCommandList) {
        unsafe {
            self.ExecuteBundle(bundle);
        }
    }

    fn set_descriptor_heaps(&self, heaps: &[DescriptorHeap]) {
        unsafe {
            self.SetDescriptorHeaps(
                heaps.len() as _,
                heaps.as_ptr() as *mut &DescriptorHeap as *mut _,
            );
        }
    }

    fn set_compute_root_signature(&self, signature: RootSignature) {
        unsafe {
            self.SetComputeRootSignature(signature);
        }
    }

    fn set_graphics_root_signature(&self, signature: RootSignature) {
        unsafe {
            self.SetGraphicsRootSignature(signature);
        }
    }

    fn set_compute_root_descriptor_table(
        &self,
        root_index: RootIndex,
        base_descriptor: GpuDescriptor,
    ) {
        unsafe {
            self.SetComputeRootDescriptorTable(root_index, base_descriptor);
        }
    }

    fn set_compute_root_constant_buffer_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    ) {
        unsafe {
            self.SetComputeRootConstantBufferView(root_index, buffer_location);
        }
    }

    fn set_compute_root_shader_resource_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    ) {
        unsafe {
            self.SetComputeRootShaderResourceView(root_index, buffer_location);
        }
    }

    fn set_compute_root_unordered_access_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    ) {
        unsafe {
            self.SetComputeRootUnorderedAccessView(root_index, buffer_location);
        }
    }

    fn set_compute_root_constant(&self, root_index: RootIndex, value: u32, dest_offset_words: u32) {
        unsafe {
            self.SetComputeRoot32BitConstant(root_index, value, dest_offset_words);
        }
    }

    fn set_graphics_root_descriptor_table(
        &self,
        root_index: RootIndex,
        base_descriptor: GpuDescriptor,
    ) {
        unsafe {
            self.SetGraphicsRootDescriptorTable(root_index, base_descriptor);
        }
    }

    fn set_graphics_root_constant_buffer_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    ) {
        unsafe {
            self.SetGraphicsRootConstantBufferView(root_index, buffer_location);
        }
    }

    fn set_graphics_root_shader_resource_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    ) {
        unsafe {
            self.SetGraphicsRootShaderResourceView(root_index, buffer_location);
        }
    }

    fn set_graphics_root_unordered_access_view(
        &self,
        root_index: RootIndex,
        buffer_location: GpuAddress,
    ) {
        unsafe {
            self.SetGraphicsRootUnorderedAccessView(root_index, buffer_location);
        }
    }

    fn set_graphics_root_constant(
        &self,
        root_index: RootIndex,
        value: u32,
        dest_offset_words: u32,
    ) {
        unsafe {
            self.SetGraphicsRoot32BitConstant(root_index, value, dest_offset_words);
        }
    }

    fn resource_barrier(&self, barriers: &[ResourceBarrier]) {
        unsafe {
            self.ResourceBarrier(barriers.len() as _, barriers.as_ptr() as _) // matches representation
        }
    }
}
