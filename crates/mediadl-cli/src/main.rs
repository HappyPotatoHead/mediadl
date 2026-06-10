use clap::{Parser, Subcommand, ValueEnum};

use mediadl_core::config::{AppConfig, default_config_path, load_or_create};
use mediadl_core::download::{
    AudioDownloadRequest, VideoDownloadRequest, download_audio, download_audio_batch_parallel,
    download_video, download_video_batch_parallel, load_batch_file,
};
use mediadl_core::validation::check_dependencies;

#[derive(Parser)]
#[command(name = "mediadl")]
#[command(version = "1.0")]
#[command(about="yt-dlp wrapper", long_about =None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
enum DownloadKind {
    Audio,
    Video,
}

#[derive(Clone, Debug, ValueEnum)]
enum ConfigKey {
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
enum ConfigCommands {
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

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    check_dependencies()?;

    // let config_path = default_config_path()?;
    // println!("Config path = {}", config_path.display());

    // let config = load_or_create()?;

    match cli.command {
        Commands::Audio {
            url,
            creator,
            collection,
            retries,
        } => {
            let config = load_or_create()?;

            let request = AudioDownloadRequest {
                url,
                creator,
                collection,
                retries,
                show_progress: None,
            };

            download_audio(request, &config)?;
        }
        Commands::Video {
            url,
            creator,
            collection,
            retries,
        } => {
            let config = load_or_create()?;

            let request = VideoDownloadRequest {
                url,
                creator,
                collection,
                retries,
                show_progress: None,
            };

            download_video(request, &config)?;
        }
        Commands::Batch { path, kind } => {
            let config = load_or_create()?;
            let entries = load_batch_file(path)?;

            match kind {
                DownloadKind::Audio => {
                    let requests: Vec<AudioDownloadRequest> = entries
                        .into_iter()
                        .map(AudioDownloadRequest::from)
                        .collect();

                    download_audio_batch_parallel(&requests, &config)?;
                }
                DownloadKind::Video => {
                    let requests: Vec<VideoDownloadRequest> = entries
                        .into_iter()
                        .map(VideoDownloadRequest::from)
                        .collect();

                    download_video_batch_parallel(&requests, &config)?;
                }
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::Show => {
                let config = load_or_create()?;
                println!("{}", config);
            }
            ConfigCommands::Reset => {
                AppConfig::reset_default_config()?;
                println!("Config reset to defaults.");
            }
            ConfigCommands::Edit { key } => {
                let mut config = load_or_create()?;

                match key {
                    Some(key) => {
                        edit_one_config_value(&mut config, key)?;
                    }
                    None => {
                        edit_full_config(&mut config)?;
                    }
                }

                let path = default_config_path()?;
                config.save_config_file(path)?;

                println!("Config saved.");
            }

            ConfigCommands::Set { key, value } => {
                let mut config = load_or_create()?;

                apply_config_value(&mut config, key, &value)?;

                let path = default_config_path()?;
                config.save_config_file(path)?;

                println!("Config saved.");
            }
        },
    }

    Ok(())
}

fn prompt_keep_existing(label: &str, current: &str) -> Result<Option<String>, String> {
    use std::io::{self, Write};

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

fn apply_config_value(config: &mut AppConfig, key: ConfigKey, value: &str) -> Result<(), String> {
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
        ConfigKey::DownloadPath => config.default_download_path.display().to_string(),
        ConfigKey::AudioFormat => config.default_audio_format.to_string(),
        ConfigKey::VideoFormat => config.default_video_format.to_string(),
        ConfigKey::VideoQuality => config.default_video_quality.to_string(),
        ConfigKey::AudioThumbnail => config.audio_thumbnail.to_string(),
        ConfigKey::VideoThumbnail => config.video_thumbnail.to_string(),
        ConfigKey::AudioOutputTemplate => config.audio_output_template.display().to_string(),
        ConfigKey::VideoOutputTemplate => config.video_output_template.display().to_string(),
        ConfigKey::Retries => config.default_retries.to_string(),
        ConfigKey::MaxParallelDownloads => config.max_parallel_downloads.to_string(),
    }
}

fn edit_one_config_value(config: &mut AppConfig, key: ConfigKey) -> Result<(), String> {
    let current = config_value(config, key.clone());

    if let Some(value) = prompt_keep_existing(&format!("{key:?}"), &current)? {
        apply_config_value(config, key, &value)?;
    }
    Ok(())
}

fn edit_full_config(config: &mut AppConfig) -> Result<(), String> {
    println!("Edit config. Press Enter to keep the current value.\n");
    println!("Optionally, you can edit the config file directly!\n");

    if let Some(value) = prompt_keep_existing(
        "Download path",
        &config.default_download_path.display().to_string(),
    )? {
        config.set_download_path(value)?;
    }

    if let Some(value) =
        prompt_keep_existing("Audio format", &config.default_audio_format.to_string())?
    {
        config.default_audio_format = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Video format", &config.default_video_format.to_string())?
    {
        config.default_video_format = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Video quality", &config.default_video_quality.to_string())?
    {
        config.default_video_quality = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Audio thumbnail", &config.audio_thumbnail.to_string())?
    {
        config.audio_thumbnail = value.parse()?;
    }

    if let Some(value) =
        prompt_keep_existing("Video thumbnail", &config.video_thumbnail.to_string())?
    {
        config.video_thumbnail = value.parse()?;
    }

    if let Some(value) = prompt_keep_existing(
        "Audio output template",
        &config.audio_output_template.display().to_string(),
    )? {
        config.set_audio_template_path(value)?;
    }

    if let Some(value) = prompt_keep_existing(
        "Video output template",
        &config.video_output_template.display().to_string(),
    )? {
        config.set_video_template_path(value)?;
    }

    if let Some(value) = prompt_keep_existing("Retries", &config.default_retries.to_string())? {
        config.set_default_retries(&value)?;
    }

    if let Some(value) = prompt_keep_existing(
        "Max parallel downloads",
        &config.max_parallel_downloads.to_string(),
    )? {
        config.set_max_parallel_downloads(&value)?;
    }

    let path = default_config_path()?;
    config.save_config_file(path)?;

    println!("\nConfig saved.");
    Ok(())
}
