//! Pipeline state

use crate::{com::WeakPtr, Blob, Error};

use std::{ops::Deref, ptr};
use windows::runtime;
use windows::Win32::Graphics::Direct3D11;
use windows::Win32::Graphics::Direct3D12;
use windows::Win32::Graphics::Hlsl;

bitflags! {
    pub struct PipelineStateFlags: u32 {
        const TOOL_DEBUG = Direct3D12::D3D12_PIPELINE_STATE_FLAG_TOOL_DEBUG.0 as u32;
    }
}

bitflags! {
    pub struct ShaderCompileFlags: u32 {
        const DEBUG = Hlsl::D3DCOMPILE_DEBUG;
        const SKIP_VALIDATION = Hlsl::D3DCOMPILE_SKIP_VALIDATION;
        const SKIP_OPTIMIZATION = Hlsl::D3DCOMPILE_SKIP_OPTIMIZATION;
        const PACK_MATRIX_ROW_MAJOR = Hlsl::D3DCOMPILE_PACK_MATRIX_ROW_MAJOR;
        const PACK_MATRIX_COLUMN_MAJOR = Hlsl::D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR;
        const PARTIAL_PRECISION = Hlsl::D3DCOMPILE_PARTIAL_PRECISION;
        // TODO: add missing flags
    }
}

#[derive(Copy, Clone)]
pub struct Shader(Direct3D12::D3D12_SHADER_BYTECODE);
impl Shader {
    pub fn null() -> Self {
        Shader(Direct3D12::D3D12_SHADER_BYTECODE {
            BytecodeLength: 0,
            pShaderBytecode: ptr::null_mut(),
        })
    }

    pub fn from_raw(data: &[u8]) -> Self {
        Shader(Direct3D12::D3D12_SHADER_BYTECODE {
            BytecodeLength: data.len() as _,
            pShaderBytecode: data.as_ptr() as _,
        })
    }

    // `blob` may not be null.
    pub fn from_blob(blob: Blob) -> Self {
        Shader(Direct3D12::D3D12_SHADER_BYTECODE {
            BytecodeLength: unsafe { blob.GetBufferSize() },
            pShaderBytecode: unsafe { blob.GetBufferPointer() },
        })
    }

    /// Compile a shader from raw HLSL.
    ///
    /// * `target`: example format: `ps_5_1`.
    pub fn compile(
        code: &[u8],
        target: &str,
        entry: &str,
        flags: ShaderCompileFlags,
    ) -> runtime::Result<(Blob, Error)> {
        let mut shader: Option<Direct3D11::ID3DBlob> = None;
        let mut error: Option<Direct3D11::ID3DBlob> = None;

        let hr = unsafe {
            Hlsl::D3DCompile(
                code.as_ptr() as *const _,
                code.len(),
                "",          // defines
                ptr::null(), // include
                None,
                entry,
                target,
                flags.bits(),
                0,
                &mut shader,
                &mut error,
            )
        };

        hr.map(|()| {
            let wk_shader = match shader {
                Some(mut blob) => unsafe { WeakPtr::from_raw(&mut blob) },
                None => WeakPtr::<Direct3D11::ID3DBlob>::null(),
            };
            let wk_err = match error {
                Some(mut err) => unsafe { WeakPtr::from_raw(&mut err) },
                None => WeakPtr::<Direct3D11::ID3DBlob>::null(),
            };
            (wk_shader, wk_err)
        })
    }
}

impl Deref for Shader {
    type Target = Direct3D12::D3D12_SHADER_BYTECODE;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Option<Blob>> for Shader {
    fn from(blob: Option<Blob>) -> Self {
        match blob {
            Some(b) => Shader::from_blob(b),
            None => Shader::null(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct CachedPSO(Direct3D12::D3D12_CACHED_PIPELINE_STATE);
impl CachedPSO {
    pub fn null() -> Self {
        CachedPSO(Direct3D12::D3D12_CACHED_PIPELINE_STATE {
            CachedBlobSizeInBytes: 0,
            pCachedBlob: ptr::null_mut(),
        })
    }

    // `blob` may not be null.
    pub fn from_blob(blob: Blob) -> Self {
        CachedPSO(Direct3D12::D3D12_CACHED_PIPELINE_STATE {
            CachedBlobSizeInBytes: unsafe { blob.GetBufferSize() },
            pCachedBlob: unsafe { blob.GetBufferPointer() },
        })
    }
}

impl Deref for CachedPSO {
    type Target = Direct3D12::D3D12_CACHED_PIPELINE_STATE;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type PipelineState = WeakPtr<Direct3D12::ID3D12PipelineState>;

#[repr(u32)]
pub enum Subobject {
    RootSignature = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE.0 as u32,
    VS = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VS.0 as u32,
    PS = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS.0 as u32,
    DS = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DS.0 as u32,
    HS = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_HS.0 as u32,
    GS = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_GS.0 as u32,
    CS = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CS.0 as u32,
    StreamOutput = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_STREAM_OUTPUT.0 as u32,
    Blend = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND.0 as u32,
    SampleMask = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK.0 as u32,
    Rasterizer = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER.0 as u32,
    DepthStencil = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL.0 as u32,
    InputLayout = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_INPUT_LAYOUT.0 as u32,
    IBStripCut = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_IB_STRIP_CUT_VALUE.0 as u32,
    PrimitiveTopology = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY.0 as u32,
    RTFormats = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS.0 as u32,
    DSFormat = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT.0 as u32,
    SampleDesc = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC.0 as u32,
    NodeMask = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_NODE_MASK.0 as u32,
    CachedPSO = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CACHED_PSO.0 as u32,
    Flags = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_FLAGS.0 as u32,
    DepthStencil1 = Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL1.0 as u32,
    // ViewInstancing = Direct3D12::D3D12__PIPELINE_STATE_SUBOBJECT_TYPE_VIEW_INSTANCING,
}

/// Subobject of a pipeline stream description
#[repr(C)]
pub struct PipelineStateSubobject<T> {
    subobject_align: [usize; 0], // Subobjects must have the same alignment as pointers.
    subobject_type: Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    subobject: T,
}

impl<T> PipelineStateSubobject<T> {
    pub fn new(subobject_type: Subobject, subobject: T) -> Self {
        PipelineStateSubobject {
            subobject_align: [],
            subobject_type: Direct3D12::D3D12_PIPELINE_STATE_SUBOBJECT_TYPE(subobject_type as _),
            subobject,
        }
    }
}
