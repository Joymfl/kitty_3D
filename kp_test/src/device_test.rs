use std::ffi::{CStr, c_void};
use std::ptr;

use libloading::Library;
extern crate khronos_egl as egl;

// 1. Define the EGL types and C function signatures manually,
// as extension function pointers aren't always explicitly wrapped in the crate.
type EGLDeviceEXT = *mut c_void;
type EGLBoolean = egl::Boolean;
type EGLint = egl::Int;

type EglQueryDevicesEXT = unsafe extern "C" fn(
    max_devices: EGLint,
    devices: *mut EGLDeviceEXT,
    num_devices: *mut EGLint,
) -> EGLBoolean;

pub fn test_device() -> Result<(), Box<dyn std::error::Error>> {
    let lib = unsafe { Library::new("libEGL.so") }.map_err(|_| egl::Error::NotInitialized)?;
    // handle nvidia registration via glvnd
    // TODO: Add support for other backends, or move to egl-wayland
    let egl = unsafe {
        egl::DynamicInstance::<egl::EGL1_5>::load_required_from(lib).expect("unable to load lib")
    };

    // 3. Load the `eglQueryDevicesEXT` extension function pointer
    let proc_name = CStr::from_bytes_with_nul(b"eglQueryDevicesEXT\0")?;
    let egl_query_devices_ext: EglQueryDevicesEXT = unsafe {
        let addr = egl.get_proc_address(proc_name.to_str()?);
        if addr.is_none() {
            return Err("EGL_EXT_device_enumeration extension is not supported".into());
        }
        std::mem::transmute(addr)
    };

    // 4. Query the number of available devices first
    let mut num_devices: EGLint = 0;
    let success = unsafe { egl_query_devices_ext(0, ptr::null_mut(), &mut num_devices) };

    if success != egl::TRUE || num_devices == 0 {
        return Err("Failed to query or found 0 EGL devices.".into());
    }

    println!("Found {} EGL device(s).", num_devices);

    // 5. Retrieve the device handles
    let mut devices: Vec<EGLDeviceEXT> = vec![ptr::null_mut(); num_devices as usize];
    let success =
        unsafe { egl_query_devices_ext(num_devices, devices.as_mut_ptr(), &mut num_devices) };

    if success == egl::TRUE {
        for (i, device) in devices.iter().enumerate() {
            println!("Device [{}]: Pointer {:?}", i, device);
        }
    }

    Ok(())
}
