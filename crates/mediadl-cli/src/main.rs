mod commands;
use clap::Parser;
use commands::{
    Cli, Commands, ConfigCommands, DownloadKind, apply_config_value, edit_full_config,
    edit_one_config_value,
};
use mediadl_core::config::{AppConfig, default_config_path, load_or_create};
use mediadl_core::download::{
    AudioDownloadRequest, VideoDownloadRequest, download_audio, download_audio_batch_parallel,
    download_video, download_video_batch_parallel, load_batch_file,
};
use mediadl_core::validation::check_dependencies;

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    check_dependencies()?;

    match cli.command {
        Commands::Audio {
            url,
            creator,
            collection,
            retries,
        } => {
            let config = load_or_create()?;

            let mut request = AudioDownloadRequest::new(url);
            request.creator = creator;
            request.collection = collection;
            request.retries = retries;

            download_audio(request, &config)?;
        }
        Commands::Video {
            url,
            creator,
            collection,
            retries,
        } => {
            let config = load_or_create()?;

            let mut request = VideoDownloadRequest::new(url);
            request.creator = creator;
            request.collection = collection;
            request.retries = retries;

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
