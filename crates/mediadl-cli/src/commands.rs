use clap::{Parser, Subcommand, ValueEnum};
use mediadl_core::config::{AppConfig, default_config_path};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "mediadl")]
#[command(version = "1.0.1")]
#[command(about="rust-based yt-dlp wrapper", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// TODO: figure out ways to combine some of these

#[derive(Subcommand)]
pub enum Commands {
    Audio {
        url: String,

        #[arg(long)]
        creator: Option<String>,

        #[arg(long)]
        collection: Option<String>,

        #[arg(long)]
        retries: Option<u8>,
    },

    Video {
        url: String,

        #[arg(long)]
        creator: Option<String>,

        #[arg(long)]
        collection: Option<String>,

        #[arg(long)]
        retries: Option<u8>,
    },

    Batch {
        path: String,

        #[arg(long = "type", value_enum)]
        kind: DownloadKind,
    },

    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DownloadKind {
    Audio,
    Video,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ConfigKey {
    DownloadPath,
    AudioFormat,
    VideoFormat,
    VideoQuality,
    AudioThumbnail,
    VideoThumbnail,
    AudioOutputTemplate,
    VideoOutputTemplate,
    Retries,
    MaxParallelDownloads,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    Show,
    Edit {
        #[arg(value_enum)]
        key: Option<ConfigKey>,
    },
    Set {
        #[arg(value_enum)]
        key: ConfigKey,
        value: String,
    },
    Reset,
}

fn prompt_keep_existing(label: &str, current: &str) -> Result<Option<String>, String> {
    print!("{label} [{current}]: ");
    io::stdout()
        .flush()
        .map_err(|err| format!("failed to flush stdout: {err}"))?;

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .map_err(|err| format!("failed to read input: {err}"))?;

    let input = input.trim();

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}

pub fn apply_config_value(
    config: &mut AppConfig,
    key: ConfigKey,
    value: &str,
) -> Result<(), String> {
    match key {
        ConfigKey::DownloadPath => config.set_download_path(value)?,
        ConfigKey::AudioFormat => config.default_audio_format = value.parse()?,
        ConfigKey::VideoFormat => config.default_video_format = value.parse()?,
        ConfigKey::VideoQuality => config.default_video_quality = value.parse()?,
        ConfigKey::AudioThumbnail => config.audio_thumbnail = value.parse()?,
        ConfigKey::VideoThumbnail => config.video_thumbnail = value.parse()?,
        ConfigKey::AudioOutputTemplate => config.set_audio_template_path(value)?,
        ConfigKey::VideoOutputTemplate => config.set_video_template_path(value)?,
        ConfigKey::Retries => config.set_default_retries(value)?,
        ConfigKey::MaxParallelDownloads => config.set_max_parallel_downloads(value)?,
    }
    Ok(())
}

fn config_value(config: &AppConfig, key: ConfigKey) -> String {
    match key {
        ConfigKey::DownloadPath => config.get_download_path().display().to_string(),
        ConfigKey::AudioFormat => config.default_audio_format.to_string(),
        ConfigKey::VideoFormat => config.default_video_format.to_string(),
        ConfigKey::VideoQuality => config.default_video_quality.to_string(),
        ConfigKey::AudioThumbnail => config.audio_thumbnail.to_string(),
        ConfigKey::VideoThumbnail => config.video_thumbnail.to_string(),
        ConfigKey::AudioOutputTemplate => config.get_audio_template_path().display().to_string(),
        ConfigKey::VideoOutputTemplate => config.get_video_template_path().display().to_string(),
        ConfigKey::Retries => config.get_default_retries().to_string(),
        ConfigKey::MaxParallelDownloads => config.get_max_parallel_downloads().to_string(),
    }
}

// DONE: remaining functions

pub fn edit_one_config_value(config: &mut AppConfig, key: ConfigKey) -> Result<(), String> {
    let current = config_value(config, key);

    if let Some(value) = prompt_keep_existing(&format!("{key:?}"), &current)? {
        apply_config_value(config, key, &value)?;
    }
    Ok(())
}

pub fn edit_full_config(config: &mut AppConfig) -> Result<(), String> {
    println!("Press Enter to keep the current value.\n");

    if let Some(value) = prompt_keep_existing(
        "Download path",
        &config.get_download_path().display().to_string(),
    )? {
        config.set_download_path(value)?;
    }

    if let Some(value) =
        prompt_keep_existing("Format (Audio)", &config.default_audio_format.to_string())?
    {
        config.default_audio_format = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Format (Video)", &config.default_video_format.to_string())?
    {
        config.default_video_format = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Quality (Video)", &config.default_video_quality.to_string())?
    {
        config.default_video_quality = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Thumbnail (Audio)", &config.audio_thumbnail.to_string())?
    {
        config.audio_thumbnail = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Thumbnail (Video)", &config.video_thumbnail.to_string())?
    {
        config.video_thumbnail = value.parse()?;
    }

    if let Some(value) = prompt_keep_existing(
        "Output template (Audio)",
        &config.get_audio_template_path().display().to_string(),
    )? {
        config.set_audio_template_path(value)?;
    }

    if let Some(value) = prompt_keep_existing(
        "Output template (Video)",
        &config.get_video_template_path().display().to_string(),
    )? {
        config.set_video_template_path(value)?;
    }

    if let Some(value) = prompt_keep_existing("Retries", &config.get_default_retries().to_string())?
    {
        config.set_default_retries(value.as_str())?;
    }

    if let Some(value) = prompt_keep_existing(
        "Parallel Download (Count)",
        &config.get_max_parallel_downloads().to_string(),
    )? {
        config.set_max_parallel_downloads(value.as_str())?;
    }

    let path = default_config_path()?;
    config.save_config_file(path)?;

    Ok(())
}
