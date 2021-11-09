use crate::com::WeakPtr;
use windows::Win32::Graphics::Direct3D12;

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum QueryHeapType {
    Occlusion = Direct3D12::D3D12_QUERY_HEAP_TYPE_OCCLUSION.0 as u32,
    Timestamp = Direct3D12::D3D12_QUERY_HEAP_TYPE_TIMESTAMP.0 as u32,
    PipelineStatistics = Direct3D12::D3D12_QUERY_HEAP_TYPE_PIPELINE_STATISTICS.0 as u32,
    SOStatistics = Direct3D12::D3D12_QUERY_HEAP_TYPE_SO_STATISTICS.0 as u32,
    // VideoDecodeStatistcs = Direct3D12::D3D12__QUERY_HEAP_TYPE_VIDEO_DECODE_STATISTICS,
    // CopyQueueTimestamp = Direct3D12::D3D12__QUERY_HEAP_TYPE_COPY_QUEUE_TIMESTAMP,
}

pub type QueryHeap = WeakPtr<Direct3D12::ID3D12QueryHeap>;
