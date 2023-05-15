use std::{
    ptr,
    sync::{atomic::AtomicBool, Arc},
    thread, time,
};

use winapi::{
    ctypes::c_void,
    shared::mmreg::WAVEFORMATEX,
    um::{
        audioclient::{IAudioClient, IAudioRenderClient, AUDCLNT_BUFFERFLAGS_SILENT},
        audiosessiontypes::{AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_RATEADJUST},
        combaseapi::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL},
        mmdeviceapi::{eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator},
        objbase::COINIT_SPEED_OVER_MEMORY,
    },
    Class, Interface,
};

use crate::wav_file::WavFile;

const REFTIMES_PER_SEC: i64 = 10000000;
const REFTIMES_PER_MILLISEC: i64 = 10000;

pub struct WavPlayer {
    audio_client: *mut IAudioClient,
    render_client: *mut IAudioRenderClient,
}

impl WavPlayer {
    pub fn new() -> Self {
        let mut audio_client: *mut IAudioClient = ptr::null_mut();

        unsafe {
            let hr = CoInitializeEx(ptr::null_mut(), COINIT_SPEED_OVER_MEMORY);
            if hr < 0 {
                panic!("Failed to initialize COM!");
            }

            let mut device_enum: *mut IMMDeviceEnumerator = ptr::null_mut();
            let hr = CoCreateInstance(
                &MMDeviceEnumerator::uuidof(),
                ptr::null_mut(),
                CLSCTX_ALL,
                &IMMDeviceEnumerator::uuidof(),
                &mut device_enum as *mut *mut IMMDeviceEnumerator as *mut *mut c_void,
            );
            if hr < 0 {
                panic!("Failed to get device enumerator!");
            }

            let mut device: *mut IMMDevice = ptr::null_mut();
            let hr = (*device_enum).GetDefaultAudioEndpoint(
                eRender,
                eConsole,
                &mut device as *mut *mut IMMDevice,
            );
            if hr < 0 {
                panic!("Failed to get default audio endpoint!");
            }

            (*device_enum).Release();

            let hr = (*device).Activate(
                &IAudioClient::uuidof(),
                CLSCTX_ALL,
                ptr::null_mut(),
                &mut audio_client as *mut *mut IAudioClient as *mut *mut c_void,
            );
            if hr < 0 {
                panic!("Failed to get default audio endpoint!");
            }

            (*device).Release();
        };

        WavPlayer {
            audio_client: audio_client,
            render_client: ptr::null_mut(),
        }
    }

    pub fn play_file(&mut self, input_file: &mut WavFile, signal_flag: Arc<AtomicBool>) {
        let hns_request_duration = REFTIMES_PER_SEC;
        let mut num_frames_padding = 0;
        let mut flags: u32 = 0;

        let mut wave_format = WAVEFORMATEX {
            wFormatTag: 0,
            nChannels: 0,
            nSamplesPerSec: 0,
            nAvgBytesPerSec: 0,
            nBlockAlign: 0,
            wBitsPerSample: 0,
            cbSize: 0,
        };

        input_file.set_format(&mut wave_format);

        unsafe {
            let hr = (*self.audio_client).Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_RATEADJUST,
                hns_request_duration,
                0,
                &wave_format,
                ptr::null_mut(),
            );
            if hr < 0 {
                panic!("Failed to initialize audio client!");
            }

            let mut buffer_frame_count = 0;
            let hr = (*self.audio_client).GetBufferSize(&mut buffer_frame_count);
            if hr < 0 {
                panic!("Failed to get buffer size!");
            }

            let hr = (*self.audio_client).GetService(
                &IAudioRenderClient::uuidof(),
                &mut self.render_client as *mut *mut IAudioRenderClient as *mut *mut c_void,
            );
            if hr < 0 {
                panic!("Failed to get render client!");
            }

            let mut p_data: *mut u8 = ptr::null_mut();
            let hr = (*self.render_client).GetBuffer(buffer_frame_count, &mut p_data);
            if hr < 0 {
                panic!("Failed to get render buffer!");
            }

            input_file.load_data(buffer_frame_count, p_data, &mut flags);

            let hr = (*self.render_client).ReleaseBuffer(buffer_frame_count, flags);
            if hr < 0 {
                panic!("Failed to release render buffer!");
            }

            let hns_actual_duration =
                REFTIMES_PER_SEC * buffer_frame_count as i64 / wave_format.nSamplesPerSec as i64;

            let hr = (*self.audio_client).Start();
            if hr < 0 {
                panic!("Failed to start playing!");
            }

            while flags != AUDCLNT_BUFFERFLAGS_SILENT
                && signal_flag.load(std::sync::atomic::Ordering::SeqCst)
            {
                thread::sleep(time::Duration::from_millis(
                    (hns_actual_duration / REFTIMES_PER_MILLISEC / 2) as u64,
                ));

                let hr = (*self.audio_client).GetCurrentPadding(&mut num_frames_padding);
                if hr < 0 {
                    panic!("Failed to get padding of render buffer!");
                }

                let num_frames_available = buffer_frame_count - num_frames_padding;

                let hr = (*self.render_client)
                    .GetBuffer(num_frames_available, &mut p_data as *mut *mut u8);
                if hr < 0 {
                    panic!("Failed to get render buffer!");
                }

                input_file.load_data(num_frames_available, p_data, &mut flags);

                let hr = (*self.render_client).ReleaseBuffer(num_frames_available, flags);
                if hr < 0 {
                    panic!("Failed to release render buffer!");
                }
            }

            thread::sleep(time::Duration::from_millis(
                (hns_actual_duration / REFTIMES_PER_MILLISEC / 2) as u64,
            ));

            let hr = (*self.audio_client).Stop();
            if hr < 0 {
                panic!("Failed to stop playing!");
            }
        }
    }

    pub fn play_continously(&mut self, input_file: &mut WavFile, signal_flag: Arc<AtomicBool>) {
        let hns_request_duration = REFTIMES_PER_SEC * 2;
        let mut num_frames_padding = 0;

        let mut wave_format = WAVEFORMATEX {
            wFormatTag: 0,
            nChannels: 0,
            nSamplesPerSec: 0,
            nAvgBytesPerSec: 0,
            nBlockAlign: 0,
            wBitsPerSample: 0,
            cbSize: 0,
        };

        input_file.set_format(&mut wave_format);

        unsafe {
            let hr = (*self.audio_client).Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_RATEADJUST,
                hns_request_duration,
                0,
                &wave_format,
                ptr::null_mut(),
            );
            if hr < 0 {
                panic!("Failed to initialize audio client!");
            }

            let mut buffer_frame_count = 0;
            let hr = (*self.audio_client).GetBufferSize(&mut buffer_frame_count);
            if hr < 0 {
                panic!("Failed to get buffer size!");
            }

            let hr = (*self.audio_client).GetService(
                &IAudioRenderClient::uuidof(),
                &mut self.render_client as *mut *mut IAudioRenderClient as *mut *mut c_void,
            );
            if hr < 0 {
                panic!("Failed to get render client!");
            }

            let mut p_data: *mut u8 = ptr::null_mut();

            let hr = (*self.audio_client).Start();
            if hr < 0 {
                panic!("Failed to start playing!");
            }

            while signal_flag.load(std::sync::atomic::Ordering::SeqCst) {
                let hr = (*self.audio_client).GetCurrentPadding(&mut num_frames_padding);
                if hr < 0 {
                    panic!("Failed to get padding of render buffer!");
                }

                let num_frames_available = buffer_frame_count - num_frames_padding;

                let hr = (*self.render_client)
                    .GetBuffer(num_frames_available, &mut p_data as *mut *mut u8);
                if hr < 0 {
                    panic!("Failed to get render buffer!");
                }

                input_file.load_data_continously(num_frames_available, p_data);

                let hr = (*self.render_client).ReleaseBuffer(num_frames_available, 0);
                if hr < 0 {
                    panic!("Failed to release render buffer!");
                }
            }
        }
    }
}

impl Drop for WavPlayer {
    fn drop(&mut self) {
        unsafe {
            (*self.audio_client).Release();
            (*self.render_client).Release();
            CoUninitialize();
        }
    }
}
