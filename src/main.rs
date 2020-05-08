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
mod snapshot;

use d3d::D3D11Device;
use roapi::{ro_initialize, RoInitType};
use snapshot::CaptureSnapshot;
use winapi::um::winuser::{GetDesktopWindow, MonitorFromWindow, MONITOR_DEFAULTTOPRIMARY};

fn run() -> winrt::Result<()> {
    ro_initialize(RoInitType::MultiThreaded)?;

    println!("Getting the capture item...");
    let monitor = unsafe { MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) };
    let item = capture::create_capture_item_for_monitor(monitor)?;

    println!("Setting up d3d...");
    let d3d_device = D3D11Device::new()?;
    let device = d3d_device.to_direct3d_device()?;

    println!("Taking snapshot...");
    let surface = CaptureSnapshot::take(&device, &item)?;

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
