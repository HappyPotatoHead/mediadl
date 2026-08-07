use std::cmp;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use directories::ProjectDirs;

// honestly, i don't know why i wrote enums
// it just looked clean i guess
// will add more or change the code when it becomes unscalable

#[derive(Debug, Default)]
pub enum AudioFormat {
    // I already have #[default] here, so the Default impl is not necessary
    #[default]
    Mp3,
    Opus,
    Flac,
    M4a,
    Aac,
    Custom(String),
}
#[derive(Debug, Default)]
pub enum VideoFormat {
    #[default]
    Mp4,
    Webm,
    Mkv,
    Custom(String),
}
#[derive(Debug, Default)]
pub enum VideoQuality {
    Max,
    Res1440p,
    Res1080p,
    #[default]
    Res720p,
    Res480p,
    Custom(String),
}
#[derive(Debug, Default)]
pub enum ThumbnailOption {
    #[default]
    None,
    Write,
    Embed,
    WriteAndEmbed,
}

// NOTE: rules of public and private fields
// 1. Always default to private
// 2. Important questions to ask
//      - Can I break my struct by putting a bad/invalid value in this field
//      - Am I 100% sure I'll never want to intervene when this changes, ever in the future?
// 3. If a field must stay in sync with another field, both are private
// 4. Expose behaviour, not state

#[derive(Debug)]
pub struct AppConfig {
    default_download_path: PathBuf,
    audio_output_template: PathBuf,
    video_output_template: PathBuf,
    default_retries: u8,
    max_parallel_downloads: u8,

    pub default_audio_format: AudioFormat,
    pub default_video_format: VideoFormat,
    pub default_video_quality: VideoQuality,
    pub video_thumbnail: ThumbnailOption,
    pub audio_thumbnail: ThumbnailOption,
}

impl AppConfig {
    pub fn get_download_path(&self) -> &Path {
        &self.default_download_path
    }

    pub fn get_audio_template_path(&self) -> &Path {
        &self.audio_output_template
    }

    pub fn get_video_template_path(&self) -> &Path {
        &self.video_output_template
    }

    pub fn get_default_retries(&self) -> u8 {
        self.default_retries
    }

    pub fn get_max_parallel_downloads(&self) -> u8 {
        self.max_parallel_downloads
    }

    pub fn set_download_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path: &Path = path.as_ref();

        if path.as_os_str().is_empty() {
            return Err("Download destination cannot be empty!".to_string());
        }

        self.default_download_path = path.to_path_buf();

        Ok(())
    }

    pub fn set_audio_template_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path: &Path = path.as_ref();
        validate_output_template(path)?;
        self.audio_output_template = path.to_path_buf();
        Ok(())
    }

    pub fn set_video_template_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path: &Path = path.as_ref();
        validate_output_template(path)?;
        self.video_output_template = path.to_path_buf();
        Ok(())
    }

    pub fn set_default_retries(&mut self, value: &str) -> Result<(), String> {
        let max_retries = 10;

        let retries = value
            .trim()
            .parse::<u8>()
            .map_err(|_| format!("invalid retry count: {}", value))?;

        self.default_retries = cmp::min(retries, max_retries);

        Ok(())
    }

    pub fn set_max_parallel_downloads(&mut self, value: &str) -> Result<(), String> {
        let maximum: u8 = 5;

        let max_parallel = value
            .trim()
            .parse::<u8>()
            .map_err(|_| format!("invalid max parallel downloads: {}", value))?;

        if max_parallel == 0 {
            return Err("set_max_parallel_downloads must be at least 1".to_string());
        }

        self.max_parallel_downloads = cmp::min(maximum, max_parallel);

        Ok(())
    }

    pub fn ensure_download_path_exists(&self) -> Result<(), String> {
        fs::create_dir_all(&self.default_download_path)
            .map_err(|err| format!("failed to create download directory: {}", err))?;
        Ok(())
    }

    pub fn save_config_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path: &Path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create config directory: {}", err))?;
        }

        let content = self.to_string();

        fs::write(path, content)
            .map_err(|err| format!("failed to write to config file: {}", err))?;

        Ok(())
    }

    pub fn load_config_file<P: AsRef<Path>>(path: P) -> Result<AppConfig, String> {
        let content: String = fs::read_to_string(path)
            .map_err(|err| format!("failed to read config file: {}", err))?;

        let mut config = AppConfig::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid config line: {}", line))?;
            let key = key.trim();
            let value = value.trim();

            match key {
                "download_path" => config.set_download_path(value)?,
                "audio_format" => config.default_audio_format = value.parse::<AudioFormat>()?,
                "video_format" => config.default_video_format = value.parse::<VideoFormat>()?,
                "video_quality" => config.default_video_quality = value.parse::<VideoQuality>()?,
                "audio_thumbnail" => config.audio_thumbnail = value.parse::<ThumbnailOption>()?,
                "video_thumbnail" => config.video_thumbnail = value.parse::<ThumbnailOption>()?,
                "audio_output_template" => config.set_audio_template_path(value)?,
                "video_output_template" => config.set_video_template_path(value)?,
                "retries" => config.set_default_retries(value)?,
                "max_parallel_downloads" => config.set_max_parallel_downloads(value)?,
                other => {
                    return Err(format!("unknown config key: {}", other));
                }
            }
        }

        Ok(config)
    }

    pub fn reset_default_config() -> Result<AppConfig, String> {
        let path: PathBuf = default_config_path()?;
        AppConfig::reset_config(path)
    }

    fn reset_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, String> {
        let config = AppConfig::default();
        config.save_config_file(path)?;

        Ok(config)
    }

    // for the rat
    pub fn set_by_key(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "download_path" => self.set_download_path(value),
            "audio_format" => {
                self.default_audio_format = value.parse::<AudioFormat>()?;
                Ok(())
            }
            "video_format" => {
                self.default_video_format = value.parse::<VideoFormat>()?;
                Ok(())
            }
            "video_quality" => {
                self.default_video_quality = value.parse::<VideoQuality>()?;
                Ok(())
            }
            "audio_thumbnail" => {
                self.audio_thumbnail = value.parse::<ThumbnailOption>()?;
                Ok(())
            }
            "video_thumbnail" => {
                self.video_thumbnail = value.parse::<ThumbnailOption>()?;
                Ok(())
            }
            "audio_output_template" => self.set_audio_template_path(value),
            "video_output_template" => self.set_video_template_path(value),
            "retries" => self.set_default_retries(value),
            "max_parallel_downloads" => self.set_max_parallel_downloads(value),
            other => Err(format!("unknown config key: {}", other)),
        }
    }

    pub fn get_by_key(&self, key: &str) -> String {
        match key {
            "download_path" => self.default_download_path.display().to_string(),
            "audio_format" => self.default_audio_format.to_string(),
            "video_format" => self.default_video_format.to_string(),
            "video_quality" => self.default_video_quality.to_string(),
            "audio_thumbnail" => self.audio_thumbnail.to_string(),
            "video_thumbnail" => self.video_thumbnail.to_string(),
            "audio_output_template" => self.audio_output_template.display().to_string(),
            "video_output_template" => self.video_output_template.display().to_string(),
            "retries" => self.default_retries.to_string(),
            "max_parallel_downloads" => self.max_parallel_downloads.to_string(),
            _ => String::new(),
        }
    }
}

pub fn default_config_path() -> Result<PathBuf, String> {
    let project_dirs = ProjectDirs::from("com", "github.happypotatohead", "mediadl")
        .ok_or_else(|| "could not determine config directory".to_string())?;

    Ok(project_dirs.config_dir().join("config"))
}

pub fn load_or_create() -> Result<AppConfig, String> {
    let path = default_config_path()?;
    if path.exists() {
        return AppConfig::load_config_file(&path);
    }
    let config = AppConfig::default();
    config.save_config_file(&path)?;
    Ok(config)
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_download_path: PathBuf::from("./downloads"),
            default_audio_format: AudioFormat::default(),
            default_video_format: VideoFormat::default(),
            default_video_quality: VideoQuality::default(),
            audio_thumbnail: ThumbnailOption::default(),
            video_thumbnail: ThumbnailOption::default(),
            audio_output_template: PathBuf::from("%(title)s.%(ext)s"),
            video_output_template: PathBuf::from("%(title)s.%(ext)s"),
            default_retries: 3,
            max_parallel_downloads: 2,
        }
    }
}

impl FromStr for AudioFormat {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let cleaned = normalise_input(input, "audio format")?;

        match &*cleaned {
            "mp3" => Ok(AudioFormat::Mp3),
            "opus" => Ok(AudioFormat::Opus),
            "flac" => Ok(AudioFormat::Flac),
            "m4a" => Ok(AudioFormat::M4a),
            "aac" => Ok(AudioFormat::Aac),
            other => Ok(AudioFormat::Custom(other.to_string())),
        }
    }
}
impl FromStr for VideoFormat {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let cleaned: String = normalise_input(input, "video format")?;

        match &*cleaned {
            "mp4" => Ok(VideoFormat::Mp4),
            "mkv" => Ok(VideoFormat::Mkv),
            "webm" => Ok(VideoFormat::Webm),
            other => Ok(VideoFormat::Custom(other.to_string())),
        }
    }
}
impl FromStr for VideoQuality {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let cleaned: String = normalise_input(input, "video quality")?;

        match &*cleaned {
            "max" => Ok(VideoQuality::Max),
            "1440" | "1440p" => Ok(VideoQuality::Res1440p),
            "1080" | "1080p" => Ok(VideoQuality::Res1080p),
            "720" | "720p" => Ok(VideoQuality::Res720p),
            "480" | "480p" => Ok(VideoQuality::Res480p),
            other => Ok(VideoQuality::Custom(other.to_string())),
        }
    }
}
impl FromStr for ThumbnailOption {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let cleaned: String = normalise_input(input, "thumbnail option")?;

        match &*cleaned {
            "none" | "false" | "no" => Ok(ThumbnailOption::None),
            "write" => Ok(ThumbnailOption::Write),
            "embed" => Ok(ThumbnailOption::Embed),
            "both" | "write-and-embed" | "write_and_embed" | "true" => {
                Ok(ThumbnailOption::WriteAndEmbed)
            }
            other => Err(format!("unknown thumbnail option: {}", other)),
        }
    }
}

impl Display for AppConfig {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            format,
            "download_path={}",
            self.default_download_path.display()
        )?;
        writeln!(format, "audio_format={}", self.default_audio_format)?;
        writeln!(format, "video_format={}", self.default_video_format)?;
        writeln!(format, "video_quality={}", self.default_video_quality)?;
        writeln!(format, "audio_thumbnail={}", self.audio_thumbnail)?;
        writeln!(format, "video_thumbnail={}", self.video_thumbnail)?;
        writeln!(
            format,
            "audio_output_template={}",
            self.audio_output_template.display()
        )?;
        writeln!(
            format,
            "video_output_template={}",
            self.video_output_template.display()
        )?;
        writeln!(format, "retries={}", self.default_retries)?;
        write!(
            format,
            "max_parallel_downloads={}",
            self.max_parallel_downloads
        )
    }
}

impl Display for AudioFormat {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioFormat::Mp3 => write!(format, "mp3"),
            AudioFormat::Opus => write!(format, "opus"),
            AudioFormat::Flac => write!(format, "flac"),
            AudioFormat::M4a => write!(format, "m4a"),
            AudioFormat::Aac => write!(format, "aac"),
            AudioFormat::Custom(value) => write!(format, "{}", value),
        }
    }
}
impl Display for VideoFormat {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoFormat::Mp4 => write!(format, "mp4"),
            VideoFormat::Mkv => write!(format, "mkv"),
            VideoFormat::Webm => write!(format, "webm"),
            VideoFormat::Custom(value) => write!(format, "{}", value),
        }
    }
}
impl Display for VideoQuality {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoQuality::Max => write!(format, "max"),
            VideoQuality::Res1440p => write!(format, "1440p"),
            VideoQuality::Res1080p => write!(format, "1080p"),
            VideoQuality::Res720p => write!(format, "720p"),
            VideoQuality::Res480p => write!(format, "480p"),
            VideoQuality::Custom(value) => write!(format, "{}", value),
        }
    }
}
impl Display for ThumbnailOption {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThumbnailOption::None => write!(format, "none"),
            ThumbnailOption::Embed => write!(format, "embed"),
            ThumbnailOption::Write => write!(format, "write"),
            ThumbnailOption::WriteAndEmbed => write!(format, "both"),
        }
    }
}

fn normalise_input(input: &str, field_name: &str) -> Result<String, String> {
    let cleaned = input.trim().to_lowercase();

    if cleaned.is_empty() {
        return Err(format!("{} cannot be empty!", field_name));
    }
    Ok(cleaned)
}

fn validate_output_template(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Output template cannot be empty!".to_string());
    }

    if path.is_absolute() {
        return Err("Output template must be relative to the download path".to_string());
    }

    let template = path.to_string_lossy();

    if !template.contains("%(ext)s") {
        return Err("Output template must include %(ext)s.".to_string());
    }

    Ok(())
}

// DUMP:
//
// impl Default for AudioFormat {
//     fn default() -> Self {
//         AudioFormat::Mp3
//     }
// }
// impl Default for VideoFormat {
//     fn default() -> Self {
//         VideoFormat::Mp4
//     }
// }
// impl Default for VideoQuality {
//     fn default() -> Self {
//         VideoQuality::Res1080p
//     }
// }
// impl Default for ThumbnailOption {
//     fn default() -> Self {
//         ThumbnailOption::None
//     }
// }

// config.rs, bottom of the file
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn default_config_has_expected_download_path() {
        let config = AppConfig::default();
        assert_eq!(config.get_download_path(), Path::new("./downloads"));
    }

    #[test]
    fn set_default_retries_caps_at_max() {
        let mut config = AppConfig::default();
        config.set_default_retries("99").unwrap();
        assert_eq!(config.get_default_retries(), 10);
    }

    #[test]
    fn set_default_retries_rejects_non_numeric() {
        let mut config = AppConfig::default();
        assert!(config.set_default_retries("abc").is_err());
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = env::temp_dir().join("mediadl_test_config");
        let path = dir.join("config");

        let mut config = AppConfig::default();
        config.set_default_retries("5").unwrap();
        config.save_config_file(&path).unwrap();

        let loaded = AppConfig::load_config_file(&path).unwrap();
        assert_eq!(loaded.get_default_retries(), 5);

        let _ = std::fs::remove_dir_all(dir);
    }
}
