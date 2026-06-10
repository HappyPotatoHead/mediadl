use std::fs::File;
use std::path::Path;
use std::process::Command;

use crate::config::{AppConfig, ThumbnailOption};
use crate::validation::{normalise_url, sanitise_filename};

// the values changes every download,
// that's why i have them separate from the config
#[derive(Clone, Debug)]
pub struct AudioDownloadRequest {
    pub url: String,
    pub creator: Option<String>,
    pub collection: Option<String>,
    pub retries: Option<u8>,
    pub show_progress: Option<bool>,
}
#[derive(Clone, Debug)]
pub struct VideoDownloadRequest {
    pub url: String,
    pub creator: Option<String>,
    pub collection: Option<String>,
    pub retries: Option<u8>,
    pub show_progress: Option<bool>,
}
#[derive(Clone, Debug)]
pub struct BatchEntry {
    pub url: String,
    pub creator: Option<String>,
    pub collection: Option<String>,
}

impl AudioDownloadRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            creator: None,
            collection: None,
            retries: None,
            show_progress: None,
        }
    }
}

impl VideoDownloadRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            creator: None,
            collection: None,
            retries: None,
            show_progress: None,
        }
    }
}

impl From<BatchEntry> for AudioDownloadRequest {
    fn from(entry: BatchEntry) -> Self {
        Self {
            url: entry.url,
            creator: entry.creator,
            collection: entry.collection,
            retries: None,
            show_progress: Some(false),
        }
    }
}

impl From<BatchEntry> for VideoDownloadRequest {
    fn from(entry: BatchEntry) -> Self {
        Self {
            url: entry.url,
            creator: entry.creator,
            collection: entry.collection,
            retries: None,
            show_progress: Some(false),
        }
    }
}

pub fn download_audio(request: AudioDownloadRequest, config: &AppConfig) -> Result<(), String> {
    let normalised_url = normalise_url(&request.url)?;
    config.ensure_download_path_exists()?;

    let retries = request.retries.unwrap_or(config.default_retries);
    let show_progress = request.show_progress.unwrap_or(true);
    let attempts = retries + 1;
    let mut output_template = build_output_base_path(
        config,
        request.creator.as_deref(),
        request.collection.as_deref(),
    );
    output_template = output_template.join(&config.audio_output_template);

    run_command_with_retries(
        || {
            let mut command = Command::new("yt-dlp");
            command
                .arg("-x")
                .arg("--audio-format")
                .arg(config.default_audio_format.to_string())
                .arg("-o")
                .arg(&output_template);

            apply_thumbnail_args(&mut command, &config.audio_thumbnail);

            command.arg(&normalised_url);
            if !show_progress {
                command.arg("--no-progress");
            }
            command
        },
        attempts,
    )
}

pub fn download_video(request: VideoDownloadRequest, config: &AppConfig) -> Result<(), String> {
    let normalised_url = normalise_url(&request.url)?;
    config.ensure_download_path_exists()?;

    let retries = request.retries.unwrap_or(config.default_retries);
    let show_progress = request.show_progress.unwrap_or(true);
    let attempts = retries + 1;
    let mut output_template = build_output_base_path(
        config,
        request.creator.as_deref(),
        request.collection.as_deref(),
    );
    output_template = output_template.join(&config.video_output_template);
    let quality = config.default_video_quality.to_string();
    let format_selector = build_video_format_selector(&quality)?;

    run_command_with_retries(
        || {
            let mut command = Command::new("yt-dlp");
            command
                .arg("-f")
                .arg(&format_selector)
                .arg("--merge-output-format")
                .arg(config.default_video_format.to_string())
                .arg("-o")
                .arg(&output_template);

            apply_thumbnail_args(&mut command, &config.video_thumbnail);

            command.arg(&normalised_url);
            if !show_progress {
                command.arg("--no-progress");
            }
            command
        },
        attempts,
    )
}

pub fn load_batch_file<P: AsRef<Path>>(path: P) -> Result<Vec<BatchEntry>, String> {
    let file = File::open(path).map_err(|err| format!("failed to open batch file: {}", err))?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut entries: Vec<BatchEntry> = Vec::new();

    for result in reader.records() {
        let record = result.map_err(|err| format!("failed to read CSV record: {}", err))?;

        let url = record
            .get(0)
            .ok_or_else(|| "missing URL field".to_string())?
            .trim();

        if url.is_empty() {
            return Err("URL field cannot be empty".to_string());
        }

        let creator = record
            .get(1)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let collection = record
            .get(2)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        entries.push(BatchEntry {
            url: url.to_string(),
            creator,
            collection,
        });
    }
    Ok(entries)
}

pub fn download_audio_batch_parallel(
    requests: &[AudioDownloadRequest],
    config: &AppConfig,
) -> Result<(), String> {
    download_batch_parallel(requests, config, download_audio)
}

pub fn download_video_batch_parallel(
    requests: &[VideoDownloadRequest],
    config: &AppConfig,
) -> Result<(), String> {
    download_batch_parallel(requests, config, download_video)
}

fn build_output_base_path(
    config: &AppConfig,
    creator: Option<&str>,
    collection: Option<&str>,
) -> std::path::PathBuf {
    let mut output_template = config.default_download_path.clone();

    if let Some(creator) = creator {
        output_template = output_template.join(sanitise_filename(creator));
    }

    if let Some(collection) = collection {
        output_template = output_template.join(sanitise_filename(collection));
    }

    output_template
}

fn build_video_format_selector(quality: &str) -> Result<String, String> {
    if quality == "max" {
        return Ok("bestvideo+bestaudio/best".to_string());
    }

    let height = quality.trim_end_matches("p");

    if height.parse::<u32>().is_err() {
        return Err(format!("invalid video quality: {}", quality));
    }

    Ok(format!(
        "bestvideo[height<={height}]+bestaudio/best[height<={height}]"
    ))
}

fn apply_thumbnail_args(command: &mut Command, option: &ThumbnailOption) {
    match option {
        ThumbnailOption::None => {}
        ThumbnailOption::Write => {
            command.arg("--write-thumbnail");
        }
        ThumbnailOption::Embed => {
            command.arg("--embed-thumbnail");
        }
        ThumbnailOption::WriteAndEmbed => {
            command.arg("--write-thumbnail");
            command.arg("--embed-thumbnail");
        }
    }
}

fn run_command_with_retries<T>(mut build_command: T, attempts: u8) -> Result<(), String>
where
    T: FnMut() -> Command,
{
    let mut last_error = String::new();

    for attempt in 1..=attempts {
        let mut command = build_command();

        match run_command(&mut command) {
            Ok(()) => return Ok(()),
            Err(err) => {
                last_error = err;

                if attempt < attempts {
                    eprintln!("Attempt {attempt}/{attempts} failed. Retrying...");
                }
            }
        }
    }

    Err(format!(
        "yt-dlp failed after {attempts} attempt(s): {last_error}"
    ))
}

fn run_command(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|err| format!("failed to run yt-dlp: {}", err))?;
    if !status.success() {
        return Err(format!("yt-dlp failed with status: {}", status));
    }
    Ok(())
}

fn download_batch_parallel<T, F>(
    requests: &[T],
    config: &AppConfig,
    download_fn: F,
) -> Result<(), String>
where
    T: Clone + Send,
    F: Fn(T, &AppConfig) -> Result<(), String> + Sync,
{
    let max_parallel = config.max_parallel_downloads.max(1);
    let mut errors = Vec::new();

    for chunk in requests.chunks(max_parallel.into()) {
        let download_fn_ref = &download_fn;

        std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .cloned()
                .map(|request| scope.spawn(move || download_fn_ref(request, config)))
                .collect();

            for handle in handles {
                match handle.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => errors.push(err),
                    Err(_) => errors.push("download thread panicked".to_string()),
                }
            }
        });
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} downloads(s) failed:\n{}",
            errors.len(),
            errors.join("\n")
        ))
    }
}
