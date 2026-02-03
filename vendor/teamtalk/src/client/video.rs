//! Video capture and transmission APIs.
use super::Client;
use crate::types::{UserId, VideoCodec, VideoFormat};
use teamtalk_sys as ffi;

/// Video capture device with supported formats.
pub struct VideoCaptureDevice {
    pub id: String,
    pub name: String,
    pub api: String,
    pub formats: Vec<VideoFormat>,
}

impl From<ffi::VideoCaptureDevice> for VideoCaptureDevice {
    fn from(d: ffi::VideoCaptureDevice) -> Self {
        let mut formats = Vec::new();
        for i in 0..(d.nVideoFormatsCount as usize).min(1024) {
            formats.push(VideoFormat::from(d.videoFormats[i]));
        }
        Self {
            id: crate::utils::strings::to_string(&d.szDeviceID),
            name: crate::utils::strings::to_string(&d.szDeviceName),
            api: crate::utils::strings::to_string(&d.szCaptureAPI),
            formats,
        }
    }
}

impl Client {
    /// Returns available video capture devices.
    pub fn get_video_capture_devices(&self) -> Vec<VideoCaptureDevice> {
        let mut count: i32 = 0;
        unsafe {
            ffi::api().TT_GetVideoCaptureDevices(std::ptr::null_mut(), &mut count);
            let mut devices = vec![std::mem::zeroed::<ffi::VideoCaptureDevice>(); count as usize];
            if ffi::api().TT_GetVideoCaptureDevices(devices.as_mut_ptr(), &mut count) == 1 {
                devices.into_iter().map(VideoCaptureDevice::from).collect()
            } else {
                vec![]
            }
        }
    }

    /// Initializes a video capture device.
    pub fn init_video_capture_device(&self, device_id: &str, format: &VideoFormat) -> bool {
        let id = crate::utils::ToTT::tt(device_id);
        let raw_fmt = format.to_ffi();
        unsafe { ffi::api().TT_InitVideoCaptureDevice(self.ptr.0, id.as_ptr(), &raw_fmt) == 1 }
    }

    /// Closes the active video capture device.
    pub fn close_video_capture_device(&self) -> bool {
        unsafe { ffi::api().TT_CloseVideoCaptureDevice(self.ptr.0) == 1 }
    }

    /// Starts video transmission.
    pub fn start_video_transmission(&self, codec: &VideoCodec) -> bool {
        unsafe { ffi::api().TT_StartVideoCaptureTransmission(self.ptr.0, &codec.to_ffi()) == 1 }
    }

    /// Stops video transmission.
    pub fn stop_video_transmission(&self) -> bool {
        unsafe { ffi::api().TT_StopVideoCaptureTransmission(self.ptr.0) == 1 }
    }

    /// Acquires the latest video frame for a user.
    pub fn acquire_video_frame(&self, user_id: UserId) -> Option<*mut ffi::VideoFrame> {
        unsafe {
            let ptr = ffi::api().TT_AcquireUserVideoCaptureFrame(self.ptr.0, user_id.0);
            if ptr.is_null() { None } else { Some(ptr) }
        }
    }

    /// Releases a previously acquired video frame.
    ///
    /// # Safety
    /// - `frame` must be a pointer returned by `acquire_video_frame`.
    /// - The frame must not be released more than once.
    /// - The pointer must not be used after release.
    pub unsafe fn release_video_frame(&self, frame: *mut ffi::VideoFrame) -> bool {
        if frame.is_null() {
            return false;
        }
        unsafe { ffi::api().TT_ReleaseUserVideoCaptureFrame(self.ptr.0, frame) == 1 }
    }
}
