use crate::windows::system::DispatcherQueueController;
use winrt::RuntimeType;

#[link(name = "windowsapp")]
extern "stdcall" {
    fn RoInitialize(init_type: RoInitType) -> winrt::ErrorCode;
}

#[allow(dead_code)]
#[repr(i32)]
pub enum RoInitType {
    MultiThreaded = 0,
    SingleThreaded = 1,
}

pub fn ro_initialize(init_type: RoInitType) -> winrt::Result<()> {
    unsafe { RoInitialize(init_type).ok() }
}

#[link(name = "coremessaging")]
extern "stdcall" {
    fn CreateDispatcherQueueController(
        options: DispatcherQueueOptions,
        dispatcherQueueController: *mut <DispatcherQueueController as RuntimeType>::Abi,
    ) -> winrt::ErrorCode;
}

#[repr(C)]
struct DispatcherQueueOptions {
    size: u32,
    thread_type: DispatcherQueueThreadType,
    apartment_type: DispatcherQueueThreadApartmentType,
}

#[allow(dead_code)]
#[repr(i32)]
pub enum DispatcherQueueThreadType {
    Dedicated = 1,
    Current = 2,
}

#[allow(dead_code)]
#[repr(i32)]
pub enum DispatcherQueueThreadApartmentType {
    None = 0,
    ASTA = 1,
    STA = 2,
}

pub fn create_dispatcher_queue_controller(
    thread_type: DispatcherQueueThreadType,
    apartment_type: DispatcherQueueThreadApartmentType,
) -> winrt::Result<DispatcherQueueController> {
    let options = DispatcherQueueOptions {
        size: std::mem::size_of::<DispatcherQueueOptions>() as u32,
        thread_type,
        apartment_type,
    };
    unsafe {
        let mut result: DispatcherQueueController = std::mem::zeroed();
        CreateDispatcherQueueController(options, result.set_abi()).ok()?;
        Ok(result)
    }
}

pub fn create_dispatcher_queue_controller_for_current_thread(
) -> winrt::Result<DispatcherQueueController> {
    create_dispatcher_queue_controller(
        DispatcherQueueThreadType::Current,
        DispatcherQueueThreadApartmentType::None,
    )
}
