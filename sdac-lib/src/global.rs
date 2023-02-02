use futures::executor::block_on;

use std::borrow::BorrowMut;

use std::sync::{Mutex, Once};
use tarpc::{client, tokio_serde::formats::Json};

static mut CUDA_CLIENT: Option<Mutex<service::CudaClient>> = None;
static INIT: Once = Once::new();

pub fn client<'a>() -> &'a Mutex<service::CudaClient> {
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