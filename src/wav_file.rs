use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use winapi::{shared::mmreg::WAVEFORMATEX, um::audioclient::AUDCLNT_BUFFERFLAGS_SILENT};

struct WavHeader {
    audio_format: u16,
    num_channels: u16,
    sample_rate: u32,
    bit_depth: u16,
}

impl WavHeader {
    fn from_bytes(header_buf: &[u8]) -> Self {
        let audio_format = u16::from_le_bytes([header_buf[20], header_buf[21]]);
        let num_channels = u16::from_le_bytes([header_buf[22], header_buf[23]]);
        let sample_rate = u32::from_le_bytes([
            header_buf[24],
            header_buf[25],
            header_buf[26],
            header_buf[27],
        ]);
        let bit_depth = u16::from_le_bytes([header_buf[34], header_buf[35]]);
        WavHeader {
            audio_format,
            num_channels,
            sample_rate,
            bit_depth,
        }
    }
}
pub struct WavFile {
    file_handle: File,
    wav_header: WavHeader,
}

impl WavFile {
    pub fn new(filepath: &str) -> Self {
        let file_open_result = File::open(filepath);
        if let Err(_) = file_open_result {
            panic!("Specified wav file not found!");
        }

        let mut file_handle = file_open_result.unwrap();

        let mut wav_header_as_byte: [u8; 44] = [0; 44];
        if let Err(_) = file_handle.read_exact(&mut wav_header_as_byte) {
            panic!("Input is not in wav format!");
        }

        let wav_header = WavHeader::from_bytes(&wav_header_as_byte);

        WavFile {
            file_handle: file_handle,
            wav_header: wav_header,
        }
    }

    pub fn set_format(&self, wave_format: &mut WAVEFORMATEX) {
        wave_format.wFormatTag = self.wav_header.audio_format;
        wave_format.nChannels = self.wav_header.num_channels;
        wave_format.nSamplesPerSec = self.wav_header.sample_rate;
        wave_format.wBitsPerSample = self.wav_header.bit_depth;

        wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;

        wave_format.nAvgBytesPerSec = wave_format.nSamplesPerSec * wave_format.nBlockAlign as u32;
    }

    pub fn load_data(&mut self, buffer_frame_count: u32, p_data: *mut u8, flags: &mut u32) {
        unsafe {
            match self.file_handle.read(std::slice::from_raw_parts_mut(
                p_data,
                buffer_frame_count as usize * self.wav_header.bit_depth as usize / 8
                    * self.wav_header.num_channels as usize,
            )) {
                Ok(read_bytes) => {
                    if read_bytes < buffer_frame_count as usize {
                        if read_bytes == 0 {
                            *flags = AUDCLNT_BUFFERFLAGS_SILENT;
                            return;
                        }

                        p_data
                            .offset(read_bytes as isize)
                            .write_bytes(0, buffer_frame_count as usize - read_bytes);
                    }
                }
                _ => *flags = AUDCLNT_BUFFERFLAGS_SILENT,
            }
        }
    }

    pub fn load_data_continously(&mut self, buffer_frame_count: u32, p_data: *mut u8) {
        unsafe {
            match self.file_handle.read(std::slice::from_raw_parts_mut(
                p_data,
                buffer_frame_count as usize * self.wav_header.bit_depth as usize / 8
                    * self.wav_header.num_channels as usize,
            )) {
                Ok(read_bytes) => {
                    if read_bytes < buffer_frame_count as usize {
                        if read_bytes == 0 {
                            self.file_handle.seek(SeekFrom::Start(0)).unwrap();
                        }

                        p_data
                            .offset(read_bytes as isize)
                            .write_bytes(0, buffer_frame_count as usize - read_bytes);
                    }
                }
                _ => {
                    self.file_handle.seek(SeekFrom::Start(0)).unwrap();
                }
            }
        }
    }
}
