winrt::import!(
    dependencies
        "os"
    modules
        "windows.graphics"
        "windows.graphics.capture"
        "windows.graphics.directx"
        "windows.graphics.directx.direct3d11"
);

mod capture;
mod d3d;
mod encoder;
mod roapi;

use d3d::{D3D11Device, D3D11Texture2D};
use roapi::{ro_initialize, RoInitType};
use std::sync::mpsc::channel;
use winapi::um::d3d11::{D3D11_CPU_ACCESS_READ, D3D11_USAGE_STAGING};
use winapi::um::winuser::{GetDesktopWindow, MonitorFromWindow, MONITOR_DEFAULTTOPRIMARY};

use crate::windows::graphics::capture::Direct3D11CaptureFramePool;
use crate::windows::graphics::directx::DirectXPixelFormat;

fn run() -> winrt::Result<()> {
    type FrameArrivedHandler =
        crate::windows::foundation::TypedEventHandler<Direct3D11CaptureFramePool, winrt::Object>;
    ro_initialize(RoInitType::MultiThreaded)?;

    println!("Getting the capture item...");
    let monitor = unsafe { MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) };
    let item = capture::create_capture_item_for_monitor(monitor)?;
    let item_size = item.size()?;

    println!("Setting up d3d...");
    let d3d_device = D3D11Device::new()?;
    let d3d_context = d3d_device.get_immediate_context();
    let device = d3d_device.to_direct3d_device()?;

    println!("Setting up the frame pool...");
    let frame_pool = Direct3D11CaptureFramePool::create_free_threaded(
        &device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        &item_size,
    )?;
    let session = frame_pool.create_capture_session(&item)?;

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

    println!("Starting the capture...");
    frame_pool.frame_arrived(frame_arrived)?;
    session.start_capture()?;

    println!("We got the frame!");
    // Wait for our texture to come
    let texture = receiver.recv().unwrap();
    let surface = texture.to_direct3d_surface()?;

    println!("Saving file...");
    encoder::save_d3d_surface(&device, &surface, "screenshot.png")?;

    Ok(())
}

fn main() {
    let result = run();

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        error.code().unwrap();
    }
}
