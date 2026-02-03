use crate::types::{AudioCodec, Channel, ChannelId};
use crate::utils::ToTT;
use teamtalk_sys as ffi;

#[cfg(feature = "mock")]
pub trait TeamTalkBackend: Send + Sync {
    fn init_poll(&self) -> *mut ffi::TTInstance;
    #[cfg(windows)]
    fn init_hwnd(&self, hwnd: ffi::HWND, msg: u32) -> *mut ffi::TTInstance;
    fn close(&self, ptr: *mut ffi::TTInstance);
    fn start_recording_muxed(
        &self,
        ptr: *mut ffi::TTInstance,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool;
    fn start_recording_channel(
        &self,
        ptr: *mut ffi::TTInstance,
        channel_id: i32,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool;
    fn start_recording_streams(
        &self,
        ptr: *mut ffi::TTInstance,
        stream_types: u32,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool;
    fn stop_recording(&self, ptr: *mut ffi::TTInstance) -> bool;
    fn stop_recording_channel(&self, ptr: *mut ffi::TTInstance, channel_id: i32) -> bool;
    fn do_login_ex(
        &self,
        ptr: *mut ffi::TTInstance,
        nickname: &str,
        username: &str,
        password: &str,
        client_name: &str,
    ) -> i32;
    fn do_logout(&self, ptr: *mut ffi::TTInstance) -> i32;
    fn do_join_channel_by_id(
        &self,
        ptr: *mut ffi::TTInstance,
        channel_id: i32,
        password: &str,
    ) -> i32;
    fn do_leave_channel(&self, ptr: *mut ffi::TTInstance) -> i32;
    fn do_text_message(&self, ptr: *mut ffi::TTInstance, message: &ffi::TextMessage) -> i32;
    fn do_change_status(&self, ptr: *mut ffi::TTInstance, status_mode: i32, message: &str) -> i32;
    fn get_channel(&self, ptr: *mut ffi::TTInstance, channel_id: i32) -> Option<Channel>;
    fn get_my_user_id(&self, ptr: *mut ffi::TTInstance) -> i32;
    fn get_user(&self, ptr: *mut ffi::TTInstance, user_id: i32, user: &mut ffi::User) -> bool;
    fn get_my_channel_id(&self, ptr: *mut ffi::TTInstance) -> ChannelId;
}

#[cfg(not(feature = "mock"))]
pub(crate) trait TeamTalkBackend: Send + Sync {
    fn init_poll(&self) -> *mut ffi::TTInstance;
    #[cfg(windows)]
    fn init_hwnd(&self, hwnd: ffi::HWND, msg: u32) -> *mut ffi::TTInstance;
    fn close(&self, ptr: *mut ffi::TTInstance);
    fn start_recording_muxed(
        &self,
        ptr: *mut ffi::TTInstance,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool;
    fn start_recording_channel(
        &self,
        ptr: *mut ffi::TTInstance,
        channel_id: i32,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool;
    fn start_recording_streams(
        &self,
        ptr: *mut ffi::TTInstance,
        stream_types: u32,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool;
    fn stop_recording(&self, ptr: *mut ffi::TTInstance) -> bool;
    fn stop_recording_channel(&self, ptr: *mut ffi::TTInstance, channel_id: i32) -> bool;
    fn do_login_ex(
        &self,
        ptr: *mut ffi::TTInstance,
        nickname: &str,
        username: &str,
        password: &str,
        client_name: &str,
    ) -> i32;
    fn do_logout(&self, ptr: *mut ffi::TTInstance) -> i32;
    fn do_join_channel_by_id(
        &self,
        ptr: *mut ffi::TTInstance,
        channel_id: i32,
        password: &str,
    ) -> i32;
    fn do_leave_channel(&self, ptr: *mut ffi::TTInstance) -> i32;
    fn do_text_message(&self, ptr: *mut ffi::TTInstance, message: &ffi::TextMessage) -> i32;
    fn do_change_status(&self, ptr: *mut ffi::TTInstance, status_mode: i32, message: &str) -> i32;
    fn get_channel(&self, ptr: *mut ffi::TTInstance, channel_id: i32) -> Option<Channel>;
    fn get_my_user_id(&self, ptr: *mut ffi::TTInstance) -> i32;
    fn get_user(&self, ptr: *mut ffi::TTInstance, user_id: i32, user: &mut ffi::User) -> bool;
    fn get_my_channel_id(&self, ptr: *mut ffi::TTInstance) -> ChannelId;
}

pub(crate) struct FfiBackend;

impl TeamTalkBackend for FfiBackend {
    fn init_poll(&self) -> *mut ffi::TTInstance {
        unsafe { ffi::api().TT_InitTeamTalkPoll() }
    }

    #[cfg(windows)]
    fn init_hwnd(&self, hwnd: ffi::HWND, msg: u32) -> *mut ffi::TTInstance {
        unsafe { ffi::api().TT_InitTeamTalk(hwnd, msg) }
    }

    fn close(&self, ptr: *mut ffi::TTInstance) {
        unsafe {
            ffi::api().TT_CloseTeamTalk(ptr);
        }
    }

    fn start_recording_muxed(
        &self,
        ptr: *mut ffi::TTInstance,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        let p = file_path.tt();
        let raw_codec = codec.to_ffi();
        unsafe {
            ffi::api().TT_StartRecordingMuxedAudioFile(ptr, &raw_codec, p.as_ptr(), format) == 1
        }
    }

    fn start_recording_channel(
        &self,
        ptr: *mut ffi::TTInstance,
        channel_id: i32,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        let p = file_path.tt();
        unsafe {
            ffi::api().TT_StartRecordingMuxedAudioFileEx(ptr, channel_id, p.as_ptr(), format) == 1
        }
    }

    fn start_recording_streams(
        &self,
        ptr: *mut ffi::TTInstance,
        stream_types: u32,
        codec: &AudioCodec,
        file_path: &str,
        format: ffi::AudioFileFormat,
    ) -> bool {
        let p = file_path.tt();
        let raw_codec = codec.to_ffi();
        unsafe {
            ffi::api().TT_StartRecordingMuxedStreams(
                ptr,
                stream_types,
                &raw_codec,
                p.as_ptr(),
                format,
            ) == 1
        }
    }

    fn stop_recording(&self, ptr: *mut ffi::TTInstance) -> bool {
        unsafe { ffi::api().TT_StopRecordingMuxedAudioFile(ptr) == 1 }
    }

    fn stop_recording_channel(&self, ptr: *mut ffi::TTInstance, channel_id: i32) -> bool {
        unsafe { ffi::api().TT_StopRecordingMuxedAudioFileEx(ptr, channel_id) == 1 }
    }

    fn do_login_ex(
        &self,
        ptr: *mut ffi::TTInstance,
        nickname: &str,
        username: &str,
        password: &str,
        client_name: &str,
    ) -> i32 {
        unsafe {
            ffi::api().TT_DoLoginEx(
                ptr,
                nickname.tt().as_ptr(),
                username.tt().as_ptr(),
                password.tt().as_ptr(),
                client_name.tt().as_ptr(),
            )
        }
    }

    fn do_logout(&self, ptr: *mut ffi::TTInstance) -> i32 {
        unsafe { ffi::api().TT_DoLogout(ptr) }
    }

    fn do_join_channel_by_id(
        &self,
        ptr: *mut ffi::TTInstance,
        channel_id: i32,
        password: &str,
    ) -> i32 {
        unsafe { ffi::api().TT_DoJoinChannelByID(ptr, channel_id, password.tt().as_ptr()) }
    }

    fn do_leave_channel(&self, ptr: *mut ffi::TTInstance) -> i32 {
        unsafe { ffi::api().TT_DoLeaveChannel(ptr) }
    }

    fn do_text_message(&self, ptr: *mut ffi::TTInstance, message: &ffi::TextMessage) -> i32 {
        unsafe { ffi::api().TT_DoTextMessage(ptr, message) }
    }

    fn do_change_status(&self, ptr: *mut ffi::TTInstance, status_mode: i32, message: &str) -> i32 {
        unsafe { ffi::api().TT_DoChangeStatus(ptr, status_mode, message.tt().as_ptr()) }
    }

    fn get_channel(&self, ptr: *mut ffi::TTInstance, channel_id: i32) -> Option<Channel> {
        let mut raw = unsafe { std::mem::zeroed::<ffi::Channel>() };
        if unsafe { ffi::api().TT_GetChannel(ptr, channel_id, &mut raw) } == 1 {
            Some(Channel::from(raw))
        } else {
            None
        }
    }

    fn get_my_user_id(&self, ptr: *mut ffi::TTInstance) -> i32 {
        unsafe { ffi::api().TT_GetMyUserID(ptr) }
    }

    fn get_user(&self, ptr: *mut ffi::TTInstance, user_id: i32, user: &mut ffi::User) -> bool {
        unsafe { ffi::api().TT_GetUser(ptr, user_id, user) == 1 }
    }

    fn get_my_channel_id(&self, ptr: *mut ffi::TTInstance) -> ChannelId {
        ChannelId(unsafe { ffi::api().TT_GetMyChannelID(ptr) })
    }
}

#[cfg(feature = "mock")]
#[derive(Default)]
pub struct MockBackend {
    state: std::sync::Mutex<MockBackendState>,
}

#[cfg(feature = "mock")]
#[derive(Default)]
struct MockBackendState {
    channels: std::collections::HashMap<i32, Channel>,
    my_channel_id: ChannelId,
    my_user_id: i32,
    user: Option<ffi::User>,
    start_ok: bool,
    stop_ok: bool,
    login_result: i32,
    logout_result: i32,
    join_result: i32,
    leave_result: i32,
    last_login: Option<(String, String, String, String)>,
    last_text_message: Option<ffi::TextMessage>,
    last_status: Option<(i32, String)>,
}

#[cfg(feature = "mock")]
impl MockBackend {
    pub fn new() -> Self {
        Self {
            state: std::sync::Mutex::new(MockBackendState {
                start_ok: true,
                stop_ok: true,
                login_result: 1,
                logout_result: 1,
                join_result: 1,
                leave_result: 1,
                ..MockBackendState::default()
            }),
        }
    }

    pub fn set_channel(&self, channel: Channel) {
        let mut state = self.state.lock().unwrap();
        state.channels.insert(channel.id.0, channel);
    }

    pub fn set_my_channel_id(&self, channel_id: ChannelId) {
        let mut state = self.state.lock().unwrap();
        state.my_channel_id = channel_id;
    }

    pub fn set_my_user_id(&self, user_id: i32) {
        let mut state = self.state.lock().unwrap();
        state.my_user_id = user_id;
    }

    pub fn set_user(&self, user: ffi::User) {
        let mut state = self.state.lock().unwrap();
        state.user = Some(user);
    }

    pub fn set_start_ok(&self, ok: bool) {
        let mut state = self.state.lock().unwrap();
        state.start_ok = ok;
    }

    pub fn set_stop_ok(&self, ok: bool) {
        let mut state = self.state.lock().unwrap();
        state.stop_ok = ok;
    }

    pub fn set_login_result(&self, cmd_id: i32) {
        let mut state = self.state.lock().unwrap();
        state.login_result = cmd_id;
    }

    pub fn set_logout_result(&self, cmd_id: i32) {
        let mut state = self.state.lock().unwrap();
        state.logout_result = cmd_id;
    }

    pub fn set_join_result(&self, cmd_id: i32) {
        let mut state = self.state.lock().unwrap();
        state.join_result = cmd_id;
    }

    pub fn set_leave_result(&self, cmd_id: i32) {
        let mut state = self.state.lock().unwrap();
        state.leave_result = cmd_id;
    }

    pub fn last_login(&self) -> Option<(String, String, String, String)> {
        self.state.lock().unwrap().last_login.clone()
    }

    pub fn last_text_message(&self) -> Option<ffi::TextMessage> {
        self.state.lock().unwrap().last_text_message
    }

    pub fn last_status(&self) -> Option<(i32, String)> {
        self.state.lock().unwrap().last_status.clone()
    }
}

#[cfg(feature = "mock")]
impl TeamTalkBackend for MockBackend {
    fn init_poll(&self) -> *mut ffi::TTInstance {
        std::ptr::dangling_mut()
    }

    #[cfg(windows)]
    fn init_hwnd(&self, _hwnd: ffi::HWND, _msg: u32) -> *mut ffi::TTInstance {
        self.init_poll()
    }

    fn close(&self, ptr: *mut ffi::TTInstance) {
        let _ = ptr;
    }

    fn start_recording_muxed(
        &self,
        _ptr: *mut ffi::TTInstance,
        _codec: &AudioCodec,
        _file_path: &str,
        _format: ffi::AudioFileFormat,
    ) -> bool {
        self.state.lock().unwrap().start_ok
    }

    fn start_recording_channel(
        &self,
        _ptr: *mut ffi::TTInstance,
        _channel_id: i32,
        _file_path: &str,
        _format: ffi::AudioFileFormat,
    ) -> bool {
        self.state.lock().unwrap().start_ok
    }

    fn start_recording_streams(
        &self,
        _ptr: *mut ffi::TTInstance,
        _stream_types: u32,
        _codec: &AudioCodec,
        _file_path: &str,
        _format: ffi::AudioFileFormat,
    ) -> bool {
        self.state.lock().unwrap().start_ok
    }

    fn stop_recording(&self, _ptr: *mut ffi::TTInstance) -> bool {
        self.state.lock().unwrap().stop_ok
    }

    fn stop_recording_channel(&self, _ptr: *mut ffi::TTInstance, _channel_id: i32) -> bool {
        self.state.lock().unwrap().stop_ok
    }

    fn do_login_ex(
        &self,
        _ptr: *mut ffi::TTInstance,
        nickname: &str,
        username: &str,
        password: &str,
        client_name: &str,
    ) -> i32 {
        let mut state = self.state.lock().unwrap();
        state.last_login = Some((
            nickname.to_string(),
            username.to_string(),
            password.to_string(),
            client_name.to_string(),
        ));
        state.login_result
    }

    fn do_logout(&self, _ptr: *mut ffi::TTInstance) -> i32 {
        self.state.lock().unwrap().logout_result
    }

    fn do_join_channel_by_id(
        &self,
        _ptr: *mut ffi::TTInstance,
        _channel_id: i32,
        _password: &str,
    ) -> i32 {
        self.state.lock().unwrap().join_result
    }

    fn do_leave_channel(&self, _ptr: *mut ffi::TTInstance) -> i32 {
        self.state.lock().unwrap().leave_result
    }

    fn do_text_message(&self, _ptr: *mut ffi::TTInstance, message: &ffi::TextMessage) -> i32 {
        let mut state = self.state.lock().unwrap();
        state.last_text_message = Some(*message);
        1
    }

    fn do_change_status(&self, _ptr: *mut ffi::TTInstance, status_mode: i32, message: &str) -> i32 {
        let mut state = self.state.lock().unwrap();
        state.last_status = Some((status_mode, message.to_string()));
        1
    }

    fn get_channel(&self, _ptr: *mut ffi::TTInstance, channel_id: i32) -> Option<Channel> {
        let state = self.state.lock().unwrap();
        state.channels.get(&channel_id).cloned()
    }

    fn get_my_user_id(&self, _ptr: *mut ffi::TTInstance) -> i32 {
        self.state.lock().unwrap().my_user_id
    }

    fn get_user(&self, _ptr: *mut ffi::TTInstance, _user_id: i32, user: &mut ffi::User) -> bool {
        let state = self.state.lock().unwrap();
        if let Some(raw) = state.user {
            *user = raw;
            true
        } else {
            false
        }
    }

    fn get_my_channel_id(&self, _ptr: *mut ffi::TTInstance) -> ChannelId {
        self.state.lock().unwrap().my_channel_id
    }
}
