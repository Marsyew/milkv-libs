#![allow(non_camel_case_types, dead_code)]

use libc::{c_char, c_int, c_uchar, c_void};
use std::ffi::{CStr, CString}; 
use std::ptr;
use std::time::{Duration, Instant};  


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
    pub fn tdl_rtsp_start(
        params: *const TDL_RTSP_Params,
        out_handle: *mut TDL_RTSP_Handle,
    ) -> c_int;

    pub fn tdl_rtsp_is_running(handle: TDL_RTSP_Handle) -> c_int;
    
    pub fn tdl_rtsp_last_error(handle: TDL_RTSP_Handle) -> *const c_char;
    
    pub fn tdl_rtsp_stop(handle: TDL_RTSP_Handle) -> c_int;

    pub fn tdl_rtsp_destroy(handle: TDL_RTSP_Handle) -> c_int;
}


#[cfg(riscv_mode)]
#[link(name = "milkv_stream")]
extern "C" {
    pub fn tdl_stream_start_encoded(
        params: *const TDL_RTSP_Params,
        out_handle: *mut TDL_RTSP_Handle,
    ) -> c_int;
    
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
pub unsafe fn tdl_rtsp_start(
    _params: *const TDL_RTSP_Params,
    _out_handle: *mut TDL_RTSP_Handle,
) -> c_int {
    TDL_RTSP_ERR_GENERAL
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
pub unsafe fn tdl_rtsp_stop(_handle: TDL_RTSP_Handle) -> c_int {
    TDL_RTSP_OK  
}

#[cfg(not(riscv_mode))]
pub unsafe fn tdl_rtsp_destroy(_handle: TDL_RTSP_Handle) -> c_int {
    TDL_RTSP_OK 
}

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

pub mod rtsp {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RunState {
        Running,
        Stopped,
        Invalid,
    }


    #[derive(Debug, Clone)]
    pub struct RtspParams {
        pub rtsp_port: u16,
        pub enc_width: u32,
        pub enc_height: u32,
        pub framerate: u32,
        pub vb_blk_count: u32,
        pub vb_bind: bool,
        pub codec: String,
        pub ring_capacity: u32,
    }

    impl Default for RtspParams {
        fn default() -> Self {
            Self {
                rtsp_port: 8554,
                enc_width: 0,
                enc_height: 0,
                framerate: 30,
                vb_blk_count: 32,
                vb_bind: true,
                codec: "h264".to_string(),
                ring_capacity: 0,
            }
        }
    }

    impl RtspParams {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn port(mut self, port: u16) -> Self {
            self.rtsp_port = port;
            self
        }

        pub fn resolution(mut self, width: u32, height: u32) -> Self {
            self.enc_width = width;
            self.enc_height = height;
            self
        }

        pub fn framerate(mut self, fps: u32) -> Self {
            self.framerate = fps;
            self
        }

        pub fn codec(mut self, codec: &str) -> Self {
            self.codec = codec.to_string();
            self
        }

        pub fn vb_blocks(mut self, count: u32) -> Self {
            self.vb_blk_count = count;
            self
        }

        pub fn vb_bind(mut self, bind: bool) -> Self {
            self.vb_bind = bind;
            self
        }
    }

    pub struct RtspServer {
        handle: TDL_RTSP_Handle,
        _codec: CString,
    }

    unsafe impl Send for RtspServer {}
    unsafe impl Sync for RtspServer {}

    impl RtspServer {

        pub fn start(params: RtspParams) -> Result<Self, String> {
            let codec_c = CString::new(params.codec.as_str())
                .map_err(|e| format!("Invalid codec string: {}", e))?;

            let c_params = TDL_RTSP_Params {
                rtsp_port: params.rtsp_port,
                enc_width: params.enc_width,
                enc_height: params.enc_height,
                framerate: params.framerate,
                vb_blk_count: params.vb_blk_count,
                vb_bind: if params.vb_bind { 1 } else { 0 },
                codec: codec_c.as_ptr(),
                ring_capacity: params.ring_capacity,
            };

            unsafe {
                let mut handle: TDL_RTSP_Handle = ptr::null_mut();
                let ret = tdl_rtsp_start(&c_params, &mut handle);
                
                if ret != TDL_RTSP_OK || handle.is_null() {
                    let err_msg = if !handle.is_null() {
                        let err_ptr = tdl_rtsp_last_error(handle);
                        if !err_ptr.is_null() {
                            CStr::from_ptr(err_ptr).to_string_lossy().to_string()
                        } else {
                            format!("rtsp_start failed with code {}", ret)
                        }
                    } else {
                        format!("rtsp_start failed with code {} (null handle)", ret)
                    };
                    
                    if !handle.is_null() {
                        let _ = tdl_rtsp_destroy(handle);
                    }
                    return Err(err_msg);
                }

                Ok(Self {
                    handle,
                    _codec: codec_c,
                })
            }
        }

        pub fn wait_running(&self, timeout_ms: u64) -> bool {
            let deadline = Instant::now() + Duration::from_millis(timeout_ms);
            
            while Instant::now() < deadline {
                match self.state() {
                    RunState::Running => return true,
                    RunState::Stopped | RunState::Invalid => {}
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            
            matches!(self.state(), RunState::Running)
        }

        pub fn is_running(&self) -> bool {
            matches!(self.state(), RunState::Running)
        }

        pub fn state(&self) -> RunState {
            if self.handle.is_null() {
                return RunState::Invalid;
            }

            unsafe {
                match tdl_rtsp_is_running(self.handle) {
                    1 => RunState::Running,
                    0 => RunState::Stopped,
                    _ => RunState::Invalid,
                }
            }
        }

        pub fn last_error(&self) -> String {
            if self.handle.is_null() {
                return "NULL_HANDLE".to_string();
            }

            unsafe {
                let err_ptr = tdl_rtsp_last_error(self.handle);
                if err_ptr.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(err_ptr).to_string_lossy().to_string()
                }
            }
        }

        pub fn stop(&self) -> Result<(), String> {
            if self.handle.is_null() {
                return Ok(());
            }

            unsafe {
                let ret = tdl_rtsp_stop(self.handle);
                if ret == TDL_RTSP_OK {
                    Ok(())
                } else {
                    Err(format!(
                        "stop failed with code {}: {}",
                        ret,
                        self.last_error()
                    ))
                }
            }
        }

        pub fn raw_handle(&self) -> TDL_RTSP_Handle {
            self.handle
        }
    }

    impl Drop for RtspServer {
        fn drop(&mut self) {
            if !self.handle.is_null() {
                unsafe {
                    let _ = tdl_rtsp_stop(self.handle);
                    std::thread::sleep(Duration::from_millis(100));
                    let _ = tdl_rtsp_destroy(self.handle);
                    self.handle = ptr::null_mut();
                }
            }
        }
    }
}


pub mod stream {
    use super::*;

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
