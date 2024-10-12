use std::{
    io::{Read, Seek, SeekFrom},
    ops::DerefMut,
};

use ctru::{
    linear::LinearAllocator,
    services::ndsp::{
        wave::{Status, Wave},
        AudioFormat, AudioMix, InterpolationType, Ndsp, OutputMode,
    },
};

use crate::{Error, Result};

pub const SAMPLES_PER_BUF: usize = 44100 * 4;

pub struct PcmData<R: Read + Seek + Sized> {
    samples: R,
    sample_rate: f32,
    loop_points: Option<[u64; 2]>,

    waves: [Wave<Box<[u8], LinearAllocator>>; 2],
    active_buffer: usize,
    finished_reading: bool,
}

impl<R: Read + Seek + Sized> PcmData<R> {
    pub fn new(mut samples: R, sample_rate: f32, loop_points: Option<[u64; 2]>) -> Result<Self> {
        let waves = [(); 2].map(|_| {
            let audio_data: Box<[_], _> = Box::new_in([0u8; SAMPLES_PER_BUF], LinearAllocator);

            Wave::new(audio_data, AudioFormat::PCM16Stereo, false)
        });

        if let Some([loop_start, loop_end]) = loop_points {
            let stream_len = samples.stream_len()?;
            if loop_start > stream_len || loop_end > stream_len {
                Err(Error::Other(format!(
                    "Loop points for PCM audio incorrect (size is {}, loop points are {}-{}",
                    stream_len, loop_start, loop_end
                )))?
            }
        }

        Ok(Self {
            samples,
            sample_rate,
            loop_points,

            waves,
            active_buffer: 0,
            finished_reading: false,
        })
    }

    pub fn init(&mut self, ndsp: &mut Ndsp) -> Result<()> {
        ndsp.set_output_mode(OutputMode::Stereo);

        let mut channel = ndsp.channel(0)?;
        channel.reset();
        self.active_buffer = 0;
        self.samples.seek(SeekFrom::Start(0))?;

        channel.set_format(AudioFormat::PCM16Stereo);
        channel.set_interpolation(InterpolationType::Linear);
        channel.set_sample_rate(self.sample_rate);

        let mut mix = AudioMix::zeroed();
        mix.set_front(1.0, 1.0);

        channel.set_mix(&mix);

        for wave in 0..self.waves.len() {
            self.read_samples::<SAMPLES_PER_BUF>(wave)?;
        }

        channel.queue_wave(&mut self.waves[0])?;
        if !self.finished_reading {
            channel.queue_wave(&mut self.waves[1])?;
        }

        channel.set_paused(false);

        Ok(())
    }

    pub fn reload_buffers(&mut self, ndsp: &Ndsp) -> Result<()> {
        if self.finished_reading {
            return Ok(());
        }

        if self.waves[self.active_buffer].status() == Status::Done {
            self.read_samples::<SAMPLES_PER_BUF>(self.active_buffer)?;
            self.active_buffer = if self.active_buffer == 0 { 1 } else { 0 };
            ndsp.channel(0)?.queue_wave(&mut self.waves[self.active_buffer])?;
        }

        Ok(())
    }

    fn read_samples<const N: usize>(&mut self, wave: usize) -> Result<()> {
        if self.finished_reading {
            return Ok(());
        }

        let cur_position = self.samples.stream_position()?;

        let wave_buffer = self.waves[wave].get_buffer_mut()?;

        //TODO: figure out exactly how the loop stuff works (inclusive vs exclusive)

        #[allow(clippy::collapsible_else_if)]
        if let Some([loop_start, loop_end]) = self.loop_points {
            if cur_position + N as u64 > loop_end {
                let samples_left = (cur_position + N as u64 - loop_end) as usize;

                let mut temp_buf = Vec::with_capacity(samples_left);
                self.samples.read_exact(temp_buf.deref_mut())?;

                wave_buffer[..samples_left].copy_from_slice(&temp_buf);

                self.samples.seek(SeekFrom::Start(loop_start))?;

                let mut temp_buf = Vec::with_capacity(N - samples_left);
                self.samples.read_exact(temp_buf.deref_mut())?;

                wave_buffer[samples_left..].copy_from_slice(&temp_buf);
            } else {
                self.samples.read_exact(wave_buffer)?;
            }
        } else {
            if cur_position + N as u64 > self.samples.stream_len()? {
                let samples_left = (cur_position + N as u64 - self.samples.stream_len()?) as usize;

                let mut temp_buf = Vec::with_capacity(samples_left);
                self.samples.read_exact(temp_buf.deref_mut())?;

                wave_buffer[..samples_left].copy_from_slice(&temp_buf);

                for c in wave_buffer.iter_mut().skip(samples_left) {
                    *c = 0;
                }

                self.finished_reading = true;
            } else {
                self.samples.read_exact(wave_buffer)?;
            }
        }

        Ok(())
    }
}
