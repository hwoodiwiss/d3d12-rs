use crate::{com::WeakPtr, Blob, D3DResult, Error, TextureAddressMode};
use std::{fmt, mem, ops::Range};
use windows::Win32::Graphics::{Direct3D12, Dxgi};

pub type CpuDescriptor = Direct3D12::D3D12_CPU_DESCRIPTOR_HANDLE;
pub type GpuDescriptor = Direct3D12::D3D12_GPU_DESCRIPTOR_HANDLE;

#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub space: u32,
    pub register: u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DescriptorHeapType {
    CbvSrvUav = Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV.0 as u32,
    Sampler = Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER.0 as u32,
    Rtv = Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV.0 as u32,
    Dsv = Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE_DSV.0 as u32,
}

bitflags! {
    pub struct DescriptorHeapFlags: u32 {
        const SHADER_VISIBLE = Direct3D12::D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE.0 as u32;
    }
}

pub type DescriptorHeap = WeakPtr<Direct3D12::ID3D12DescriptorHeap>;

impl DescriptorHeap {
    pub fn start_cpu_descriptor(&self) -> CpuDescriptor {
        unsafe { self.GetCPUDescriptorHandleForHeapStart() }
    }

    pub fn start_gpu_descriptor(&self) -> GpuDescriptor {
        unsafe { self.GetGPUDescriptorHandleForHeapStart() }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ShaderVisibility {
    All = Direct3D12::D3D12_SHADER_VISIBILITY_ALL.0 as u32,
    VS = Direct3D12::D3D12_SHADER_VISIBILITY_VERTEX.0 as u32,
    HS = Direct3D12::D3D12_SHADER_VISIBILITY_HULL.0 as u32,
    DS = Direct3D12::D3D12_SHADER_VISIBILITY_DOMAIN.0 as u32,
    GS = Direct3D12::D3D12_SHADER_VISIBILITY_GEOMETRY.0 as u32,
    PS = Direct3D12::D3D12_SHADER_VISIBILITY_PIXEL.0 as u32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DescriptorRangeType {
    SRV = Direct3D12::D3D12_DESCRIPTOR_RANGE_TYPE_SRV.0 as u32,
    UAV = Direct3D12::D3D12_DESCRIPTOR_RANGE_TYPE_UAV.0 as u32,
    CBV = Direct3D12::D3D12_DESCRIPTOR_RANGE_TYPE_CBV.0 as u32,
    Sampler = Direct3D12::D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER.0 as u32,
}

#[repr(transparent)]
pub struct DescriptorRange(Direct3D12::D3D12_DESCRIPTOR_RANGE);
impl DescriptorRange {
    pub fn new(ty: DescriptorRangeType, count: u32, base_binding: Binding, offset: u32) -> Self {
        DescriptorRange(Direct3D12::D3D12_DESCRIPTOR_RANGE {
            RangeType: Direct3D12::D3D12_DESCRIPTOR_RANGE_TYPE(ty as _),
            NumDescriptors: count,
            BaseShaderRegister: base_binding.register,
            RegisterSpace: base_binding.space,
            OffsetInDescriptorsFromTableStart: offset,
        })
    }
}

impl fmt::Debug for DescriptorRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("DescriptorRange")
            .field("range_type", &self.0.RangeType)
            .field("num", &self.0.NumDescriptors)
            .field("register_space", &self.0.RegisterSpace)
            .field("base_register", &self.0.BaseShaderRegister)
            .field("table_offset", &self.0.OffsetInDescriptorsFromTableStart)
            .finish()
    }
}

#[repr(transparent)]
pub struct RootParameter(Direct3D12::D3D12_ROOT_PARAMETER);
impl RootParameter {
    // TODO: DescriptorRange must outlive Self
    pub fn descriptor_table(visibility: ShaderVisibility, ranges: &[DescriptorRange]) -> Self {
        let mut param = Direct3D12::D3D12_ROOT_PARAMETER {
            ParameterType: Direct3D12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            ShaderVisibility: Direct3D12::D3D12_SHADER_VISIBILITY(visibility as _),
            ..unsafe { mem::zeroed() }
        };

        param.Anonymous.DescriptorTable = Direct3D12::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: ranges.len() as _,
            pDescriptorRanges: ranges.iter().map(|x| x.0).collect::<Vec<_>>().as_mut_ptr(),
        };

        RootParameter(param)
    }

    pub fn constants(visibility: ShaderVisibility, binding: Binding, num: u32) -> Self {
        let mut param = Direct3D12::D3D12_ROOT_PARAMETER {
            ParameterType: Direct3D12::D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            ShaderVisibility: Direct3D12::D3D12_SHADER_VISIBILITY(visibility as _),
            ..unsafe { mem::zeroed() }
        };

        param.Anonymous.Constants = Direct3D12::D3D12_ROOT_CONSTANTS {
            ShaderRegister: binding.register,
            RegisterSpace: binding.space,
            Num32BitValues: num,
        };

        RootParameter(param)
    }

    //TODO: should this be unsafe?
    pub fn descriptor(
        ty: Direct3D12::D3D12_ROOT_PARAMETER_TYPE,
        visibility: ShaderVisibility,
        binding: Binding,
    ) -> Self {
        let mut param = Direct3D12::D3D12_ROOT_PARAMETER {
            ParameterType: ty,
            ShaderVisibility: Direct3D12::D3D12_SHADER_VISIBILITY(visibility as _),
            ..unsafe { mem::zeroed() }
        };

        param.Anonymous.Descriptor = Direct3D12::D3D12_ROOT_DESCRIPTOR {
            ShaderRegister: binding.register,
            RegisterSpace: binding.space,
        };

        RootParameter(param)
    }

    pub fn cbv_descriptor(visibility: ShaderVisibility, binding: Binding) -> Self {
        Self::descriptor(
            Direct3D12::D3D12_ROOT_PARAMETER_TYPE_CBV,
            visibility,
            binding,
        )
    }

    pub fn srv_descriptor(visibility: ShaderVisibility, binding: Binding) -> Self {
        Self::descriptor(
            Direct3D12::D3D12_ROOT_PARAMETER_TYPE_SRV,
            visibility,
            binding,
        )
    }

    pub fn uav_descriptor(visibility: ShaderVisibility, binding: Binding) -> Self {
        Self::descriptor(
            Direct3D12::D3D12_ROOT_PARAMETER_TYPE_UAV,
            visibility,
            binding,
        )
    }
}

impl fmt::Debug for RootParameter {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        enum Inner<'a> {
            Table(&'a [DescriptorRange]),
            Constants { binding: Binding, num: u32 },
            SingleCbv(Binding),
            SingleSrv(Binding),
            SingleUav(Binding),
        }
        let kind = match self.0.ParameterType {
            Direct3D12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE => unsafe {
                let raw = self.0.Anonymous.DescriptorTable;
                Inner::Table(std::slice::from_raw_parts(
                    raw.pDescriptorRanges as *const _,
                    raw.NumDescriptorRanges as usize,
                ))
            },
            Direct3D12::D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS => unsafe {
                let raw = self.0.Anonymous.Constants;
                Inner::Constants {
                    binding: Binding {
                        space: raw.RegisterSpace,
                        register: raw.ShaderRegister,
                    },
                    num: raw.Num32BitValues,
                }
            },
            _ => unsafe {
                let raw = self.0.Anonymous.Descriptor;
                let binding = Binding {
                    space: raw.RegisterSpace,
                    register: raw.ShaderRegister,
                };
                match self.0.ParameterType {
                    Direct3D12::D3D12_ROOT_PARAMETER_TYPE_CBV => Inner::SingleCbv(binding),
                    Direct3D12::D3D12_ROOT_PARAMETER_TYPE_SRV => Inner::SingleSrv(binding),
                    Direct3D12::D3D12_ROOT_PARAMETER_TYPE_UAV => Inner::SingleUav(binding),
                    other => panic!("Unexpected type {:?}", other),
                }
            },
        };

        formatter
            .debug_struct("RootParameter")
            .field("visibility", &self.0.ShaderVisibility)
            .field("kind", &kind)
            .finish()
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum StaticBorderColor {
    TransparentBlack = Direct3D12::D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK.0 as u32,
    OpaqueBlack = Direct3D12::D3D12_STATIC_BORDER_COLOR_OPAQUE_BLACK.0 as u32,
    OpaqueWhite = Direct3D12::D3D12_STATIC_BORDER_COLOR_OPAQUE_WHITE.0 as u32,
}

#[repr(transparent)]
pub struct StaticSampler(Direct3D12::D3D12_STATIC_SAMPLER_DESC);
impl StaticSampler {
    pub fn new(
        visibility: ShaderVisibility,
        binding: Binding,
        filter: Direct3D12::D3D12_FILTER,
        address_mode: TextureAddressMode,
        mip_lod_bias: f32,
        max_anisotropy: u32,
        comparison_op: Direct3D12::D3D12_COMPARISON_FUNC,
        border_color: StaticBorderColor,
        lod: Range<f32>,
    ) -> Self {
        StaticSampler(Direct3D12::D3D12_STATIC_SAMPLER_DESC {
            Filter: filter,
            AddressU: address_mode[0],
            AddressV: address_mode[1],
            AddressW: address_mode[2],
            MipLODBias: mip_lod_bias,
            MaxAnisotropy: max_anisotropy,
            ComparisonFunc: comparison_op,
            BorderColor: Direct3D12::D3D12_STATIC_BORDER_COLOR(border_color as _),
            MinLOD: lod.start,
            MaxLOD: lod.end,
            ShaderRegister: binding.register,
            RegisterSpace: binding.space,
            ShaderVisibility: Direct3D12::D3D12_SHADER_VISIBILITY(visibility as _),
        })
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum RootSignatureVersion {
    V1_0 = Direct3D12::D3D_ROOT_SIGNATURE_VERSION_1_0.0 as u32,
    V1_1 = Direct3D12::D3D_ROOT_SIGNATURE_VERSION_1_1.0 as u32,
}

bitflags! {
    pub struct RootSignatureFlags: u32 {
        const ALLOW_IA_INPUT_LAYOUT = Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT.0 as u32;
        const DENY_VS_ROOT_ACCESS = Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_DENY_VERTEX_SHADER_ROOT_ACCESS.0 as u32;
        const DENY_HS_ROOT_ACCESS = Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS.0 as u32;
        const DENY_DS_ROOT_ACCESS = Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS.0 as u32;
        const DENY_GS_ROOT_ACCESS = Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS.0 as u32;
        const DENY_PS_ROOT_ACCESS = Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_DENY_PIXEL_SHADER_ROOT_ACCESS.0 as u32;
    }
}

pub type RootSignature = WeakPtr<Direct3D12::ID3D12RootSignature>;
pub type BlobResult = D3DResult<(Blob, Error)>;

#[cfg(feature = "libloading")]
impl crate::Direct3D12Lib {
    pub fn serialize_root_signature(
        &self,
        version: RootSignatureVersion,
        parameters: &[RootParameter],
        static_samplers: &[StaticSampler],
        flags: RootSignatureFlags,
    ) -> Result<BlobResult, libloading::Error> {
        use Dxgi::ID3DBlob;
        type Fun = extern "system" fn(
            *const Direct3D12::D3D12_ROOT_SIGNATURE_DESC,
            Direct3D12::D3D_ROOT_SIGNATURE_VERSION,
            *mut *mut ID3DBlob,
            *mut *mut ID3DBlob,
        ) -> crate::HRESULT;

        let desc = Direct3D12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: parameters.len() as _,
            pParameters: parameters.as_ptr() as *const _,
            NumStaticSamplers: static_samplers.len() as _,
            pStaticSamplers: static_samplers.as_ptr() as _,
            Flags: flags.bits(),
        };

        let mut blob = Blob::null();
        let mut error = Error::null();
        let hr = unsafe {
            let func: libloading::Symbol<Fun> =
                self.lib.get(b"Direct3D12SerializeRootSignature")?;
            func(
                &desc,
                version as _,
                blob.mut_void() as *mut *mut _,
                error.mut_void() as *mut *mut _,
            )
        };

        Ok(((blob, error), hr))
    }
}

impl RootSignature {
    #[cfg(feature = "implicit-link")]
    pub fn serialize(
        version: RootSignatureVersion,
        parameters: &[RootParameter],
        static_samplers: &[StaticSampler],
        flags: RootSignatureFlags,
    ) -> BlobResult {
        let mut blob = Blob::null();
        let mut error = Error::null();

        let desc = Direct3D12::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: parameters.len() as _,
            pParameters: parameters.as_ptr() as *const _,
            NumStaticSamplers: static_samplers.len() as _,
            pStaticSamplers: static_samplers.as_ptr() as _,
            Flags: flags.bits(),
        };

        let hr = unsafe {
            Direct3D12::D3D12SerializeRootSignature(
                &desc,
                version as _,
                blob.mut_void() as *mut *mut _,
                error.mut_void() as *mut *mut _,
            )
        };

        ((blob, error), hr)
    }
}

#[repr(transparent)]
pub struct RenderTargetViewDesc(pub(crate) Direct3D12::D3D12_RENDER_TARGET_VIEW_DESC);

impl RenderTargetViewDesc {
    pub fn texture_2d(format: Dxgi::DXGI_FORMAT, mip_slice: u32, plane_slice: u32) -> Self {
        let mut desc = Direct3D12::D3D12_RENDER_TARGET_VIEW_DESC {
            Format: format,
            ViewDimension: Direct3D12::D3D12_RTV_DIMENSION_TEXTURE2D,
            ..unsafe { mem::zeroed() }
        };

        desc.Anonymous.Texture2D = Direct3D12::D3D12_TEX2D_RTV {
            MipSlice: mip_slice,
            PlaneSlice: plane_slice,
        };

        RenderTargetViewDesc(desc)
    }
}
