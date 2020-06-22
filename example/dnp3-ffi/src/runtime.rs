use crate::ffi;
use crate::*;
use dnp3::prelude::master::*;
use std::ffi::CStr;
use std::net::SocketAddr;
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::str::FromStr;
use std::time::Duration;

pub use tokio::runtime::Runtime;

fn build_runtime<F>(f: F) -> std::result::Result<tokio::runtime::Runtime, std::io::Error>
where
    F: Fn(&mut tokio::runtime::Builder) -> &mut tokio::runtime::Builder,
{
    f(tokio::runtime::Builder::new()
        .enable_all()
        .threaded_scheduler())
    .build()
}

pub(crate) unsafe fn runtime_new(
    config: *const ffi::RuntimeConfig,
) -> *mut tokio::runtime::Runtime {
    let result = match config.as_ref() {
        None => build_runtime(|r| r),
        Some(x) => build_runtime(|r| r.core_threads(x.num_core_threads as usize)),
    };

    match result {
        Ok(r) => Box::into_raw(Box::new(r)),
        Err(_) => {
            //log::error!("Unable to build runtime: {}", err);
            null_mut()
        }
    }
}

pub(crate) unsafe fn runtime_destroy(runtime: *mut tokio::runtime::Runtime) {
    if !runtime.is_null() {
        Box::from_raw(runtime);
    };
}

pub(crate) unsafe fn runtime_add_master_tcp(
    runtime: *mut tokio::runtime::Runtime,
    address: u16,
    level: ffi::DecodeLogLevel,
    strategy: ffi::ReconnectStrategy,
    response_timeout: u64,
    endpoint: *const c_char,
    listener: ffi::ClientStateListener,
) -> *mut Master {
    let strategy = ReconnectStrategy::new(
        Duration::from_millis(strategy.min_delay),
        Duration::from_millis(strategy.max_delay),
    );
    let response_timeout = Duration::from_millis(response_timeout);
    let endpoint =
        if let Ok(endpoint) = SocketAddr::from_str(&CStr::from_ptr(endpoint).to_string_lossy()) {
            endpoint
        } else {
            return std::ptr::null_mut();
        };
    let listener = ClientStateListenerAdapter::new(listener);

    let (future, handle) = create_master_tcp_client(
        address,
        level.into(),
        strategy,
        Timeout::from_duration(response_timeout).unwrap(),
        endpoint,
        listener.into_listener(),
    );

    if let Some(runtime) = runtime.as_ref() {
        runtime.spawn(future);

        let master = Master {
            runtime: runtime.handle().clone(),
            handle,
        };

        Box::into_raw(Box::new(master))
    } else {
        std::ptr::null_mut()
    }
}

unsafe impl Send for ffi::ClientStateListener {}
unsafe impl Sync for ffi::ClientStateListener {}

struct ClientStateListenerAdapter {
    native_cb: ffi::ClientStateListener,
}

impl ClientStateListenerAdapter {
    fn new(native_cb: ffi::ClientStateListener) -> Self {
        Self { native_cb }
    }

    fn into_listener(self) -> Listener<ClientState> {
        if let Some(cb) = self.native_cb.on_change {
            Listener::BoxedFn(Box::new(move |value| {
                let value = match value {
                    ClientState::Connecting => ffi::ClientState::Connecting,
                    ClientState::Connected => ffi::ClientState::Connected,
                    ClientState::WaitAfterFailedConnect(_) => {
                        ffi::ClientState::WaitAfterFailedConnect
                    }
                    ClientState::WaitAfterDisconnect(_) => ffi::ClientState::WaitAfterDisconnect,
                    ClientState::Shutdown => ffi::ClientState::Shutdown,
                };
                (cb)(value, self.native_cb.arg);
            }))
        } else {
            Listener::None
        }
    }
}

impl Drop for ClientStateListenerAdapter {
    fn drop(&mut self) {
        if let Some(cb) = self.native_cb.on_destroy {
            (cb)(self.native_cb.arg)
        }
    }
}
