#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::borrow::BorrowMut;
use std::ffi::CString;
use std::sync::{Mutex, Once};
use futures::executor::block_on;
use tarpc::{client, context, tokio_serde::formats::Json};
use service;

// TODO(asalkeld) use autogenerated code for this.
// not including everything as the externs will conflict with the definitions
// in this file. https://github.com/xertai/sdac/issues/37
pub type CUresult = ::std::os::raw::c_uint;
pub type cuuint32_t = u32;
pub type cuuint64_t = u64;
pub type CUdeviceptr_v2 = ::std::os::raw::c_ulonglong;
pub type CUdeviceptr = CUdeviceptr_v2;
pub type CUdevice_v1 = ::std::os::raw::c_int;
pub type CUdevice = CUdevice_v1;

static mut CUDA_CLIENT: Option<Mutex<service::CudaClient>> = None;
static INIT: Once = Once::new();

fn cudaClient<'a>() -> &'a Mutex<service::CudaClient> {
    INIT.call_once(|| {
        // Since this access is inside a call_once, before any other accesses, it is safe
        unsafe {
            let transport = block_on(tarpc::serde_transport::tcp::connect(
                "[::1]:50055",
                Json::default,
            ))
            .unwrap();

            // WorldClient is generated by the service attribute. It has a constructor `new` that takes a
            // config and any Transport as input.
            let client = service::CudaClient::new(client::Config::default(), transport).spawn();

            *CUDA_CLIENT.borrow_mut() = Some(Mutex::new(client));
        }
    });

    // As long as this function is the only place with access to the static variable,
    // giving out a read-only borrow here is safe because it is guaranteed no more mutable
    // references will exist at this point or in the future.
    unsafe { CUDA_CLIENT.as_ref().unwrap() }
}

#[no_mangle]
unsafe extern "C" fn cuInit(flags: ::std::os::raw::c_uint) -> CUresult {
    block_on(
        cudaClient()
            .lock()
            .unwrap()
            .cuInit(context::current(), flags),
    )
    .unwrap()
}

#[no_mangle]
unsafe extern "C" fn cuDeviceGetName(
    name: *mut ::std::os::raw::c_char,
    len: ::std::os::raw::c_int,
    dev: CUdevice,
) -> CUresult {
    let (strName, res) = block_on(cudaClient().lock().unwrap().cuDeviceGetName(
        context::current(),
        len,
        dev,
    ))
    .unwrap();

    let cs = CString::new(strName).unwrap();
    libc::strcpy(name, cs.as_ptr());

    res
}

#[no_mangle]
unsafe extern "C" fn cuDeviceGetCount(count: *mut ::std::os::raw::c_int) -> CUresult {
    let (cnt, res) = block_on(
        cudaClient()
            .lock()
            .unwrap()
            .cuDeviceGetCount(context::current()),
    )
    .unwrap();

    *count = cnt;

    res
}

#[no_mangle]
unsafe extern "C" fn cuDeviceGet(
    device: *mut CUdevice,
    ordinal: ::std::os::raw::c_int,
) -> CUresult {
    let (dev, res) = block_on(
        cudaClient()
            .lock()
            .unwrap()
            .cuDeviceGet(context::current(), ordinal),
    )
    .unwrap();

    *device = dev;

    res
}
