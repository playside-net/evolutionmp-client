use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::AtomicPtr;
use std::time::Duration;

use detour::RawDetour;
use winapi::shared::guiddef::{REFGUID, REFIID};
use winapi::shared::minwindef::FALSE;
use winapi::shared::winerror::{HRESULT_CODE, SUCCEEDED};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winnt::HRESULT;

use crate::bind_fn;
use winapi::shared::dxgi::{IDXGISwapChain, IDXGIOutput, IDXGIDevice1, IDXGIResource};
use winapi::um::d3d11::{ID3D11Device, ID3D11DeviceContext, D3D11_PRIMITIVE_TOPOLOGY, ID3D11InputLayout, ID3D11BlendState, ID3D11DepthStencilState, ID3D11RasterizerState, ID3D11ShaderResourceView, ID3D11SamplerState, ID3D11VertexShader, ID3D11ClassInstance, ID3D11Buffer, ID3D11GeometryShader, ID3D11PixelShader, ID3D11HullShader, ID3D11DomainShader, ID3D11Texture2D, ID3D11RenderTargetView};
use winapi::Interface;
use winapi::um::d3dcommon::{D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_11_0};
use winapi::shared::dxgiformat::DXGI_FORMAT;
use winapi::ctypes::c_void;


bind_fn!(GET_SWAP_CHAIN, "48 8B 05 ? ? ? ? C3 48 8B C1 8D 4A 0E", 0, () -> Option<ManuallyDrop<Box<IDXGISwapChain>>>);

macro_rules! direct_detour {
    ($name:ident,$index:ident,$repl:expr,($($arg:ty),*)->$ret:ty) => {
        lazy_static::lazy_static! {
            static ref $name: extern fn($($arg),*) -> $ret = unsafe {
                let swap_chain = GET_SWAP_CHAIN();
                let swap_chain = swap_chain.as_ref().expect("no swap chain");
                let vtable = &swap_chain.lpVtbl.read();
                let proc = vtable.$index;
                let detour = RawDetour::new(proc as _, $repl as _)
                    .expect(concat!("error detouring ", stringify!($index)));
                detour.enable().expect(concat!("error enabling detour for ", stringify!($index)));
                let trampoline = detour.trampoline() as *const ();
                std::mem::forget(detour);
                std::mem::transmute(trampoline)
            };
        }
    };
}

direct_detour!(PRESENT, Present, SwapChain::present, (&mut IDXGISwapChain, u32, u32) -> HRESULT);
direct_detour!(RESIZE_BUFFERS, ResizeBuffers, SwapChain::resize_buffers, (&mut SwapChain, u32, u32, u32, u32, u32) -> HRESULT);

static RESIZING: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static FULLSCREEN: AtomicBool = AtomicBool::new(false);


#[repr(transparent)]
struct SwapChain(IDXGISwapChain);

impl SwapChain {
    extern fn present(&mut self, sync_interval: u32, flags: u32) -> HRESULT {
        if !RESIZING.load(Ordering::Relaxed) {
            if INITIALIZED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed) == Ok(false) {
                self.initialize_devices();
            } else {
                if let Ok(fullscreen) = self.get_fullscreen_state() {
                    if FULLSCREEN.compare_exchange(!fullscreen, fullscreen, Ordering::Relaxed, Ordering::Relaxed) == Ok(!fullscreen) {
                        info!("got resized");
                        RESIZING.store(true, Ordering::SeqCst);
                        self.release_devices();
                        PRESENT(&mut self.0, sync_interval, flags);
                        RESIZING.store(false, Ordering::SeqCst);
                    }
                }
                self.draw();
            }
        }
        PRESENT(&mut self.0, sync_interval, flags)
    }

    extern fn resize_buffers(&mut self, buffer_count: u32, width: u32, height: u32, new_format: u32, flags: u32) -> HRESULT {
        info!("got buffers resized");
        RESIZING.store(true, Ordering::SeqCst);
        let result = RESIZE_BUFFERS(self, buffer_count, width, height, new_format, flags);
        RESIZING.store(false, Ordering::SeqCst);
        result
    }

    fn get_fullscreen_state(&mut self) -> Result<bool, u32> {
        let mut state = 0;
        let result = unsafe { self.0.GetFullscreenState(&mut state, std::ptr::null_mut()) };
        if SUCCEEDED(result) {
            Ok(state == 1)
        } else {
            Err(HRESULT_CODE(result) as _)
        }
    }

    fn get_resource<R: Interface, G>(&mut self, getter: G) -> Result<ManuallyDrop<Box<R>>, u32>
        where G: Fn(&mut Self, REFIID, *mut *mut c_void) -> HRESULT {

        let resource: *mut R = std::ptr::null_mut();
        let result = (getter)(self, &R::uuidof(), &mut resource.cast());
        if SUCCEEDED(result) {
            Ok(ManuallyDrop::new(unsafe { Box::from_raw(resource) }))
        } else {
            Err(HRESULT_CODE(result) as _)
        }
    }

    fn get_device<I: Interface>(&mut self) -> Result<ManuallyDrop<Box<I>>, u32> {
        self.get_resource::<I, _>(|sc, id, res| unsafe {
            sc.0.GetDevice(id, res)
        })
    }

    fn get_buffer<B: Interface>(&mut self, buffer: u32) -> Result<ManuallyDrop<Box<B>>, u32> {
        self.get_resource::<B, _>(move |sc, id, res| unsafe {
            sc.0.GetBuffer(buffer, id, res)
        })
    }

    fn get_context(&mut self) -> Option<&mut ID3D11DeviceContext> {
        let device = self.get_device::<ID3D11Device>().expect("no device found");
        let mut context = std::ptr::null_mut();
        unsafe {
            device.GetImmediateContext(&mut context);
            context.as_mut()
        }
    }

    fn create_textures(&mut self) {

    }

    fn reload_textures(&mut self) {

    }

    fn draw(&mut self) {
        self.create_textures();
        if let Ok(mut device) = self.get_device::<ID3D11Device>() {
            if let Some(context) = self.get_context() {
                let state = unsafe { DXGIState::capture(&mut *device, context) };

                warn!("frame");

                unsafe { state.restore(context); }
            } else {
                error!("missing device context");
            }
        }
    }

    fn initialize_devices(&mut self) {
        warn!("initializing dx devices...");
        if let Ok(mut device) = self.get_device::<ID3D11Device>() {
            warn!("got device {:p}", device.as_mut());
            if let Ok(mut target_texture) = self.get_buffer::<ID3D11Texture2D>(0) {
                warn!("got target texture {:p}", target_texture.as_mut());
                let target_view = self.get_resource::<ID3D11RenderTargetView, _>(|sc, id, res| unsafe {
                    device.CreateRenderTargetView(
                        std::mem::transmute(target_texture.as_ref() as *const _ as *mut ID3D11RenderTargetView),
                        std::ptr::null_mut(),
                        res.cast()
                    )
                });

                target_view.expect("failed to create target view");
                self.reload_textures();
            }
        }
    }

    fn release_devices(&mut self) {
        info!("releasing dx devices...");
    }
}

struct DXGIState {
    feature_level: D3D_FEATURE_LEVEL,
    context: *mut ID3D11DeviceContext,
    primitive_topology: D3D11_PRIMITIVE_TOPOLOGY,
    input_layout: *mut ID3D11InputLayout,
    blend_state: *mut ID3D11BlendState,
    blend_factor: [f32; 4],
    sample_mask: u32,
    depth_stencil_state: *mut ID3D11DepthStencilState,
    stencil_ref: u32,
    rasterizer_state: *mut ID3D11RasterizerState,
    pixel_shader_resource_view: *mut ID3D11ShaderResourceView,
    sampler_state: *mut ID3D11SamplerState,
    vertex_shader: *mut ID3D11VertexShader,
    vertex_shader_ci: [*mut ID3D11ClassInstance; 256],
    vertex_shader_ci_len: u32,
    vertex_shader_constant_buffer: *mut ID3D11Buffer,
    geometry_shader: *mut ID3D11GeometryShader,
    geometry_shader_ci: [*mut ID3D11ClassInstance; 256],
    geometry_shader_ci_len: u32,
    geometry_shader_constant_buffer: *mut ID3D11Buffer,
    geometry_shader_resource_view: *mut ID3D11ShaderResourceView,
    pixel_shader: *mut ID3D11PixelShader,
    pixel_shader_ci: [*mut ID3D11ClassInstance; 256],
    pixel_shader_ci_len: u32,
    hull_shader: *mut ID3D11HullShader,
    hull_shader_ci: [*mut ID3D11ClassInstance; 256],
    hull_shader_ci_len: u32,
    domain_shader: *mut ID3D11DomainShader,
    domain_shader_ci: [*mut ID3D11ClassInstance; 256],
    domain_shader_ci_len: u32,
    vertex_buffer: *mut ID3D11Buffer,
    vertex_stride: u32,
    vertex_offset: u32,
    index_buffer: *mut ID3D11Buffer,
    index_format: DXGI_FORMAT,
    index_offset: u32
}

impl DXGIState {
    unsafe fn capture(device: &ID3D11Device, context: &mut ID3D11DeviceContext) -> DXGIState {
        let mut result = DXGIState {
            feature_level: 0,
            context: std::ptr::null_mut(),
            primitive_topology: 0,
            input_layout: std::ptr::null_mut(),
            blend_state: std::ptr::null_mut(),
            blend_factor: [0.0; 4],
            sample_mask: 0,
            depth_stencil_state: std::ptr::null_mut(),
            stencil_ref: 0,
            rasterizer_state: std::ptr::null_mut(),
            pixel_shader_resource_view: std::ptr::null_mut(),
            sampler_state: std::ptr::null_mut(),
            vertex_shader: std::ptr::null_mut(),
            vertex_shader_ci: [std::ptr::null_mut(); 256],
            vertex_shader_ci_len: 0,
            vertex_shader_constant_buffer: std::ptr::null_mut(),
            geometry_shader: std::ptr::null_mut(),
            geometry_shader_ci: [std::ptr::null_mut(); 256],
            geometry_shader_ci_len: 0,
            geometry_shader_constant_buffer: std::ptr::null_mut(),
            geometry_shader_resource_view: std::ptr::null_mut(),
            pixel_shader: std::ptr::null_mut(),
            pixel_shader_ci: [std::ptr::null_mut(); 256],
            pixel_shader_ci_len: 0,
            hull_shader: std::ptr::null_mut(),
            hull_shader_ci: [std::ptr::null_mut(); 256],
            hull_shader_ci_len: 0,
            domain_shader: std::ptr::null_mut(),
            domain_shader_ci: [std::ptr::null_mut(); 256],
            domain_shader_ci_len: 0,
            vertex_buffer: std::ptr::null_mut(),
            vertex_stride: 0,
            vertex_offset: 0,
            index_buffer: std::ptr::null_mut(),
            index_format: 0,
            index_offset: 0
        };
        result.feature_level = device.GetFeatureLevel();
        context.IAGetPrimitiveTopology(&mut result.primitive_topology);
        context.IAGetInputLayout(&mut result.input_layout);
        context.OMGetBlendState(&mut result.blend_state, &mut result.blend_factor, &mut result.sample_mask);
        context.OMGetDepthStencilState(&mut result.depth_stencil_state, &mut result.stencil_ref);
        context.RSGetState(&mut result.rasterizer_state);

        result.vertex_shader_ci_len = 256;
        context.VSGetShader(&mut result.vertex_shader, result.vertex_shader_ci.as_mut_ptr(), &mut result.vertex_shader_ci_len);
        context.VSGetConstantBuffers(0, 1, &mut result.vertex_shader_constant_buffer);

        result.pixel_shader_ci_len = 256;
        context.PSGetShader(&mut result.pixel_shader, result.pixel_shader_ci.as_mut_ptr(), &mut result.pixel_shader_ci_len);
        context.PSGetShaderResources(0, 1, &mut result.pixel_shader_resource_view);
        context.PSGetSamplers(0, 1, &mut result.sampler_state);

        if result.feature_level >= D3D_FEATURE_LEVEL_10_0 {
            result.geometry_shader_ci_len = 256;
            context.GSGetShader(&mut result.geometry_shader, result.geometry_shader_ci.as_mut_ptr(), &mut result.geometry_shader_ci_len);
            context.GSGetConstantBuffers(0, 1, &mut result.geometry_shader_constant_buffer);
            context.GSGetShaderResources(0, 1, &mut result.geometry_shader_resource_view);

            if result.feature_level >= D3D_FEATURE_LEVEL_11_0 {
                result.hull_shader_ci_len = 256;
                context.HSGetShader(&mut result.hull_shader, result.hull_shader_ci.as_mut_ptr(), &mut result.hull_shader_ci_len);

                result.domain_shader_ci_len = 256;
                context.DSGetShader(&mut result.domain_shader, result.domain_shader_ci.as_mut_ptr(), &mut result.domain_shader_ci_len);
            }
        }

        context.IAGetVertexBuffers(0, 1, &mut result.vertex_buffer, &mut result.vertex_stride, &mut result.vertex_offset);
        context.IAGetIndexBuffer(&mut result.index_buffer, &mut result.index_format, &mut result.index_offset);

        result
    }

    unsafe fn restore(self, context: &mut ID3D11DeviceContext) {
        context.IASetPrimitiveTopology(self.primitive_topology);
        context.IASetInputLayout(self.input_layout);
        context.OMSetBlendState(self.blend_state, &self.blend_factor, self.sample_mask);
        context.OMSetDepthStencilState(self.depth_stencil_state, self.stencil_ref);
        context.RSSetState(self.rasterizer_state);
        context.VSSetShader(self.vertex_shader, self.vertex_shader_ci.as_ptr(), self.vertex_shader_ci_len);
        context.VSSetConstantBuffers(0, 1, &self.vertex_shader_constant_buffer);
        context.PSSetShader(self.pixel_shader, self.pixel_shader_ci.as_ptr(), self.pixel_shader_ci_len);
        context.PSSetShaderResources(0, 1, &self.pixel_shader_resource_view);
        context.PSSetSamplers(0, 1, &self.sampler_state);

        if self.feature_level >= D3D_FEATURE_LEVEL_10_0 {
            context.GSSetShader(self.geometry_shader, self.geometry_shader_ci.as_ptr(), self.geometry_shader_ci_len);
            context.GSSetConstantBuffers(0, 1, &self.geometry_shader_constant_buffer);
            context.GSSetShaderResources(0, 1, &self.geometry_shader_resource_view);

            if self.feature_level >= D3D_FEATURE_LEVEL_11_0 {
                context.HSSetShader(self.hull_shader, self.hull_shader_ci.as_ptr(), self.hull_shader_ci_len);
                context.DSSetShader(self.domain_shader, self.domain_shader_ci.as_ptr(), self.domain_shader_ci_len);
            }
        }

        context.IASetVertexBuffers(0, 1, &self.vertex_buffer, &self.vertex_stride, &self.vertex_offset);
        context.IASetIndexBuffer(self.index_buffer, self.index_format, self.index_offset);
    }
}

pub unsafe fn hook() {
    let lib = GetModuleHandleA(c_str!("d3d11.dll").as_ptr());
    assert!(!lib.is_null(), "no d3d11.dll present");
    lazy_static::initialize(&GET_SWAP_CHAIN);
    std::thread::spawn(|| {
        loop {
            if GET_SWAP_CHAIN().is_some() {
                lazy_static::initialize(&PRESENT);
                lazy_static::initialize(&RESIZE_BUFFERS);
                break;
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    });
}