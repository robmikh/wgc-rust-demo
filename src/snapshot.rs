use crate::windows::graphics::capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use crate::windows::graphics::directx::DirectXPixelFormat;
use crate::windows::graphics::directx::direct3d11::{IDirect3DDevice, IDirect3DSurface};
use crate::d3d::{D3D11Device, D3D11Texture2D};
use std::sync::mpsc::channel;
use winapi::um::d3d11::{D3D11_CPU_ACCESS_READ, D3D11_USAGE_STAGING};

type FrameArrivedHandler =
        crate::windows::foundation::TypedEventHandler<Direct3D11CaptureFramePool, winrt::Object>;

pub struct CaptureSnapshot;

impl CaptureSnapshot {
    // TODO: Allow to create non-stagingn textures
    // TODO: Allow specifying pixel format
    // TODO: Async?
    pub fn take(device: &IDirect3DDevice, item: &GraphicsCaptureItem) -> winrt::Result<IDirect3DSurface> {
        let d3d_device = D3D11Device::from_direct3d_device(device)?;
        let d3d_context = d3d_device.get_immediate_context();
        let item_size = item.size()?;

        // Initialize the capture
        let frame_pool = Direct3D11CaptureFramePool::create_free_threaded(
            device,
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            &item_size,
        )?;
        let session = frame_pool.create_capture_session(item)?;
    
        // Setup the frame arrived handler
        let (sender, receiver) = channel();
        let frame_arrived = FrameArrivedHandler::new({
            let d3d_device = d3d_device.clone();
            let d3d_context = d3d_context.clone();
            let session = session.clone();
            move |frame_pool, _| {
                let frame = frame_pool.try_get_next_frame()?;
                let surface = frame.surface()?;
    
                let frame_texture = D3D11Texture2D::from_direct3d_surface(&surface)?;
    
                // Make a copy of the texture
                let mut desc = frame_texture.get_desc();
                // Make this a staging texture
                desc.Usage = D3D11_USAGE_STAGING;
                desc.BindFlags = 0;
                desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
                desc.MiscFlags = 0;
                let copy_texture = d3d_device.create_texture_2d(&desc, None)?;
                d3d_context.copy_resource(&copy_texture, &frame_texture);
    
                // End the capture
                session.close()?;
                frame_pool.close()?;
    
                sender.send(copy_texture).unwrap();
    
                Ok(())
            }
        });
    
        // Start the capture
        frame_pool.frame_arrived(frame_arrived)?;
        session.start_capture()?;
    
        // Wait for our texture to come
        let texture = receiver.recv().unwrap();
        let surface = texture.to_direct3d_surface()?;

        Ok(surface)
    }
}