use std::{
    fs::File,
    io::BufReader,
    path::Path,
    thread,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rodio::{Decoder, OutputStreamBuilder, Sink, Source};

use crate::model::AudioClipSettings;

struct PreviewPlayback {
    clip: AudioClipSettings,
    _stream: rodio::OutputStream,
    sink: Sink,
}

static PREVIEW_PLAYBACK: Lazy<Mutex<Option<PreviewPlayback>>> = Lazy::new(|| Mutex::new(None));

pub fn load_duration_ms(path: &str) -> Result<u64> {
    let decoder = open_decoder(path)?;
    let duration = decoder
        .total_duration()
        .context("Could not determine the audio duration")?;
    Ok(duration.as_millis() as u64)
}

pub fn play_clip_async(clip: AudioClipSettings) {
    thread::spawn(move || {
        let _ = play_clip_blocking(&clip);
    });
}

pub fn play_clip_sequence_async(clips: Vec<AudioClipSettings>) {
    thread::spawn(move || {
        let _ = play_clip_sequence_blocking(&clips);
    });
}

pub fn play_clip_blocking(clip: &AudioClipSettings) -> Result<()> {
    play_clip_sequence_blocking(std::slice::from_ref(clip))
}

pub fn play_clip_sequence_blocking(clips: &[AudioClipSettings]) -> Result<()> {
    let clips = clips
        .iter()
        .filter(|clip| clip.enabled && !clip.file_path.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();
    if clips.is_empty() {
        return Ok(());
    }
    for clip in &clips {
        let path = clip.file_path.trim();
        if !Path::new(path).exists() {
            bail!("Audio file was not found");
        }
    }

    let stream = OutputStreamBuilder::open_default_stream()
        .context("Could not open the default audio output")?;
    let sink = Sink::connect_new(stream.mixer());
    for clip in &clips {
        sink.set_volume(clip.volume.clamp(0.0, 2.0));
        sink.append(clipped_source(clip)?);
    }
    sink.sleep_until_end();
    Ok(())
}

pub fn toggle_preview(mut clip: AudioClipSettings) -> Result<bool> {
    if !clip.enabled || clip.file_path.trim().is_empty() {
        bail!("Choose an audio file first");
    }
    clip.enabled = true;

    let mut playback = PREVIEW_PLAYBACK.lock();
    cleanup_preview(&mut playback);

    if playback
        .as_ref()
        .is_some_and(|current| current.clip == clip)
    {
        if let Some(current) = playback.take() {
            current.sink.stop();
        }
        return Ok(false);
    }

    if let Some(current) = playback.take() {
        current.sink.stop();
    }

    let stream = OutputStreamBuilder::open_default_stream()
        .context("Could not open the default audio output")?;
    let sink = Sink::connect_new(stream.mixer());
    sink.set_volume(clip.volume.clamp(0.0, 2.0));
    sink.append(clipped_source(&clip)?);
    sink.play();

    *playback = Some(PreviewPlayback {
        clip,
        _stream: stream,
        sink,
    });
    Ok(true)
}

pub fn stop_preview() {
    if let Some(current) = PREVIEW_PLAYBACK.lock().take() {
        current.sink.stop();
    }
}

pub fn is_previewing(clip: &AudioClipSettings) -> bool {
    let mut playback = PREVIEW_PLAYBACK.lock();
    cleanup_preview(&mut playback);
    playback
        .as_ref()
        .is_some_and(|current| current.clip == *clip)
}

pub fn load_waveform(path: &str, buckets: usize) -> Result<Vec<f32>> {
    let path = path.trim();
    if path.is_empty() {
        bail!("Choose an audio file first");
    }
    if !Path::new(path).exists() {
        bail!("Audio file was not found");
    }

    let mut decoder = open_decoder(path)?;
    let bucket_count = buckets.max(32);
    let estimated_total_samples = decoder.total_duration().map(|duration| {
        (duration.as_secs_f64() * decoder.sample_rate() as f64 * decoder.channels() as f64)
            .round() as usize
    });
    let samples_per_bucket = estimated_total_samples
        .map(|total| (total / bucket_count).max(1))
        .unwrap_or(2048);

    let mut peaks = vec![0.0f32; bucket_count];
    let mut sample_index = 0usize;
    for sample in decoder.by_ref() {
        let bucket = (sample_index / samples_per_bucket).min(bucket_count - 1);
        peaks[bucket] = peaks[bucket].max(sample.abs());
        sample_index += 1;
    }

    if sample_index == 0 {
        return Ok(peaks);
    }

    let peak_max = peaks
        .iter()
        .copied()
        .fold(0.0f32, |best, current| best.max(current));
    if peak_max > 0.0 {
        for peak in &mut peaks {
            *peak /= peak_max;
        }
    }

    Ok(peaks)
}

fn clipped_source(
    clip: &AudioClipSettings,
) -> Result<impl Source<Item = rodio::Sample> + Send + 'static> {
    let decoder = open_decoder(&clip.file_path)?;
    let start = Duration::from_millis(clip.start_ms);
    let end = Duration::from_millis(clip.end_ms.max(clip.start_ms));
    let length = end.saturating_sub(start);
    Ok(decoder
        .skip_duration(start)
        .take_duration(length)
        .speed(clip.speed.clamp(0.25, 3.0)))
}

fn open_decoder(path: &str) -> Result<Decoder<BufReader<File>>> {
    let file = File::open(path).with_context(|| format!("Failed to open audio file: {path}"))?;
    Decoder::new(BufReader::new(file)).context("Failed to decode the audio file")
}

fn cleanup_preview(playback: &mut Option<PreviewPlayback>) {
    if playback.as_ref().is_some_and(|current| current.sink.empty()) {
        *playback = None;
    }
}
