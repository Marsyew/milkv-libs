#![allow(non_camel_case_types, dead_code)]

use libc::{c_char, c_int, c_uchar, c_void};

pub type TDL_RTSP_Handle = *mut c_void;


pub const TDL_RTSP_OK: c_int = 0;
pub const TDL_RTSP_ERR_GENERAL: c_int = -1;
pub const TDL_RTSP_ERR_PARAM: c_int = -2;
pub const TDL_RTSP_ERR_STATE: c_int = -3;
pub const TDL_RTSP_ERR_INIT: c_int = -4;
pub const TDL_STREAM_ERR_TIMEOUT: c_int = -4;
pub const TDL_STREAM_ERR_BUF_SMALL: c_int = -5;

#[repr(C)]
#[derive(Debug)]
pub struct TDL_RTSP_Params {
    pub rtsp_port: u16,
    pub enc_width: u32,
    pub enc_height: u32,
    pub framerate: u32,
    pub vb_blk_count: u32,
    pub vb_bind: c_uchar,
    pub codec: *const c_char,
    pub ring_capacity: u32,
}


#[cfg(riscv_mode)]
#[link(name = "milkv_stream")]
extern "C" {
    pub fn tdl_stream_start_encoded(
        params: *const TDL_RTSP_Params,
        out_handle: *mut TDL_RTSP_Handle,
    ) -> c_int;
    
    pub fn tdl_rtsp_is_running(handle: TDL_RTSP_Handle) -> c_int;
    pub fn tdl_rtsp_last_error(handle: TDL_RTSP_Handle) -> *const c_char;
    pub fn tdl_rtsp_stop(handle: TDL_RTSP_Handle);
    pub fn tdl_rtsp_destroy(handle: TDL_RTSP_Handle);
    
    pub fn tdl_stream_get_frame(
        handle: TDL_RTSP_Handle,
        buf: *mut u8,
        inout_size: *mut u32,
        timeout_ms: c_int,
        pts: *mut u64,
        is_key: *mut c_int,
    ) -> c_int;
    
    pub fn tdl_stream_get_drop_count(handle: TDL_RTSP_Handle) -> u64;
}


#[cfg(not(riscv_mode))]
pub unsafe fn tdl_stream_start_encoded(
    _params: *const TDL_RTSP_Params,
    _out_handle: *mut TDL_RTSP_Handle,
) -> c_int {
    TDL_RTSP_ERR_GENERAL
}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_rtsp_is_running(_handle: TDL_RTSP_Handle) -> c_int {
    0
}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_rtsp_last_error(_handle: TDL_RTSP_Handle) -> *const c_char {
    b"Not supported on non-riscv platforms\0".as_ptr() as *const c_char
}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_rtsp_stop(_handle: TDL_RTSP_Handle) {}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_rtsp_destroy(_handle: TDL_RTSP_Handle) {}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_stream_get_frame(
    _handle: TDL_RTSP_Handle,
    _buf: *mut u8,
    _inout_size: *mut u32,
    _timeout_ms: c_int,
    _pts: *mut u64,
    _is_key: *mut c_int,
) -> c_int {
    TDL_STREAM_ERR_TIMEOUT
}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_stream_get_drop_count(_handle: TDL_RTSP_Handle) -> u64 {
    0
}

pub mod stream {
    use super::*;
    use std::ffi::CStr;
    use std::ptr;

    pub struct StreamHandle {
        raw: TDL_RTSP_Handle,
    }

    unsafe impl Send for StreamHandle {}
    unsafe impl Sync for StreamHandle {}

    impl StreamHandle {
        pub fn start_encode_only(params: &TDL_RTSP_Params) -> Result<Self, String> {
            unsafe {
                let mut h: TDL_RTSP_Handle = ptr::null_mut();
                let r = tdl_stream_start_encoded(params as *const _, &mut h as *mut _);
                if r != TDL_RTSP_OK || h.is_null() {
                    return Err(format!("tdl_stream_start_encoded failed r={}", r));
                }
                
                std::thread::sleep(std::time::Duration::from_millis(200));
                
                if tdl_rtsp_is_running(h) != 1 {
                    let err_msg = CStr::from_ptr(tdl_rtsp_last_error(h))
                        .to_string_lossy()
                        .into_owned();
                    tdl_rtsp_destroy(h);
                    return Err(format!("tdl_rtsp_is_running check failed: {}", err_msg));
                }
                
                Ok(Self { raw: h })
            }
        }

        pub fn get_encoded_frame(
            &self,
            timeout_ms: i32,
        ) -> Result<Option<(Vec<u8>, u64, bool)>, String> {
            unsafe {
                let mut need: u32 = 0;
                let mut rc = tdl_stream_get_frame(
                    self.raw,
                    ptr::null_mut(),
                    &mut need,
                    timeout_ms,
                    ptr::null_mut(),
                    ptr::null_mut(),
                );

                if rc == TDL_STREAM_ERR_TIMEOUT {
                    return Ok(None);
                }
                if rc == TDL_RTSP_ERR_STATE {
                    return Err("Handle stopped or invalid state".into());
                }
                if need == 0 {
                    return Ok(None);
                }

                let mut buf = vec![0u8; need as usize];
                let mut size_in = need;
                let mut pts = 0u64;
                let mut is_key_i = 0i32;
                rc = tdl_stream_get_frame(
                    self.raw,
                    buf.as_mut_ptr(),
                    &mut size_in,
                    0,
                    &mut pts,
                    &mut is_key_i,
                );

                if rc == 0 {
                    buf.truncate(size_in as usize);
                    let is_key = is_key_i != 0;
                    Ok(Some((buf, pts, is_key)))
                } else {
                    Err(format!("Fetch frame failed rc={}", rc))
                }
            }
        }

        pub fn stop(&self) {
            if !self.raw.is_null() {
                unsafe {
                    tdl_rtsp_stop(self.raw);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    impl Drop for StreamHandle {
        fn drop(&mut self) {
            if !self.raw.is_null() {
                unsafe {
                    tdl_rtsp_destroy(self.raw);
                    self.raw = ptr::null_mut();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(TDL_RTSP_OK, 0);
        assert_eq!(TDL_RTSP_ERR_GENERAL, -1);
    }
}
