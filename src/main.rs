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
mod displays;
mod encoder;
mod roapi;
mod snapshot;
mod window_finder;

use clap::{value_t, App, Arg, ArgMatches};
use d3d::D3D11Device;
use displays::enumerate_displays;
use roapi::{ro_initialize, RoInitType};
use snapshot::CaptureSnapshot;
use std::io::Write;
use winapi::um::winuser::{
    GetDesktopWindow, GetWindowThreadProcessId, MonitorFromWindow, MONITOR_DEFAULTTOPRIMARY,
};
use window_finder::find_window;

use windows::graphics::capture::GraphicsCaptureItem;

fn run(matches: &ArgMatches) -> winrt::Result<()> {
    ro_initialize(RoInitType::MultiThreaded)?;

    let item = get_capture_item_from_matches(matches)?;
    let d3d_device = D3D11Device::new()?;
    let device = d3d_device.to_direct3d_device()?;
    let surface = CaptureSnapshot::take(&device, &item)?;
    encoder::save_d3d_surface(&device, &surface, "screenshot.png")?;

    Ok(())
}

fn main() {
    let matches = App::new("wgc-rust-demo")
        .version("0.1.0")
        .author("Robert Mikhayelyan <rob.mikh@outlook.com>")
        .about("A demo that saves screenshots of windows or monitors using Windows.Graphics.Capture and Rust/WinRT.")
        .arg(Arg::with_name("window")
            .short("w")
            .long("window")
            .value_name("window title query")
            .help("Capture a window who's title contains the provided input")
            .conflicts_with_all(&["monitor", "primary"])
            .takes_value(true))
        .arg(Arg::with_name("monitor")
            .short("m")
            .long("monitor")
            .value_name("monitor number")
            .help("Capture a monitor")
            .conflicts_with_all(&["window", "primary"])
            .takes_value(true))
        .arg(Arg::with_name("primary")
            .short("p")
            .long("primary")
            .help("Capture the primary monitor (default if no params are specified)")
            .conflicts_with_all(&["window", "monitor"])
            .takes_value(false))
        .get_matches();

    let result = run(&matches);

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        error.code().unwrap();
    }
}

fn get_capture_item_from_matches(matches: &ArgMatches) -> winrt::Result<GraphicsCaptureItem> {
    let item = if matches.is_present("window") {
        let query = matches.value_of("window").unwrap();
        let windows = find_window(query);
        let window = {
            if windows.len() == 0 {
                println!("No window matching '{}' found!", query);
                std::process::exit(1);
            } else if windows.len() == 1 {
                &windows[0]
            } else {
                println!(
                    "{} windows found matching '{}', please select one:",
                    windows.len(),
                    query
                );
                println!("    Num       PID    Window Title");
                for (i, window) in windows.iter().enumerate() {
                    let mut pid = 0;
                    unsafe { GetWindowThreadProcessId(window.handle, &mut pid) };
                    println!("    {:>3}    {:>6}    {}", i, pid, window.title);
                }
                let index: usize;
                loop {
                    print!("Please make a selection (q to quit): ");
                    std::io::stdout().flush().unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    if input.to_lowercase().contains("q") {
                        std::process::exit(0);
                    }
                    let input = input.trim();
                    let selection: Option<usize> = match input.parse::<usize>() {
                        Ok(selection) => {
                            if selection < windows.len() {
                                Some(selection)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(selection) = selection {
                        index = selection;
                        break;
                    } else {
                        println!("Invalid input, '{}'!", input);
                        continue;
                    };
                }
                &windows[index]
            }
        };
        capture::create_capture_item_for_window(window.handle)?
    } else if matches.is_present("monitor") {
        let id = value_t!(matches, "monitor", u32).unwrap();
        let displays = enumerate_displays();
        if id <= 0 {
            println!("Invalid input, ids start with 1.");
            std::process::exit(1);
        }
        let index = (id - 1) as usize;
        if index >= displays.len() {
            println!("Invalid input, id is higher than the number of displays!");
            std::process::exit(1);
        }
        let display = &displays[index];
        capture::create_capture_item_for_monitor(display.handle)?
    } else {
        let monitor = unsafe { MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) };
        capture::create_capture_item_for_monitor(monitor)?
    };

    Ok(item)
}
