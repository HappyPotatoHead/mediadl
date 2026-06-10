<a id="readme-top"></a>

<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Unlicense License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <!-- <a href="https://github.com/othneildrew/Best-README-Template"> -->
  <!--   <img src="images/logo.png" alt="Logo" width="80" height="80"> -->
  <!-- </a> -->

  <h3 align="center">mediadl</h3>

  <p align="center">
    A yt-dlp wrapper for media archiving.
    <br />
    <a href="https://github.com/HappyPotatoHead/mediadl"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/HappyPotatoHead/mediadl/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/HappyPotatoHead/mediadl/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#config">Configuration</a></li>
    <li><a href="#notes">Notes</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#disclaimer">Disclaimer</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->

## About The Project

A [yt-dlp](https://github.com/yt-dlp/yt-dlp) wrapper for media archiving.

A small CLI tool built with rust.

### Built With

- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [![Rust][Rust.com]][Rust-url]
  - [crate clap](https://docs.rs/clap/latest/clap/)

<p align="right"><a href="#readme-top">⬆️</a></p>

<!-- GETTING STARTED -->

## Getting Started

### Prerequisites

mediadl requires the following tools to be installed and available in PATH:

- yt-dlp
  - Refer to [yt-dlp download guide](https://github.com/yt-dlp/yt-dlp/wiki/Installation) for your respective OS
- ffmpeg
  - Refer to [ffmpeg download page](https://ffmpeg.org/download.html) for your respective OS

```zsh
# these commands are just examples, please refer to the links above before downloading

# macOS
bew install yt-dlp ffmpeg

# Windows
winget install yt-dlp.yt-dlp
winget install Gyan.FFmpeg
```

### Installation

#### From Releases

Download the binary for your platform from GitHub Release.

#### From Source

Build binary

```zsh
git clone https://github.com/HappyPotatoHead/mediadl.git
cd mediadl
cargo build -p mediadl-cli --release
```

For local install:

```zsh
mkdir -p ~/.local/bin
cp target/release/mediadl ~/.local/bin/
```

<p align="right"><a href="#readme-top">⬆️</a></p>

<!-- USAGE EXAMPLES -->

## Usage

### Audio

With URL only

```zsh
mediadl audio "https://youtu.be/..."
```

With creator only:

```zsh
mediadl audio "https://youtu.be/.." --creator "Me"
```

With creator and collection:

```zsh
mediadl audio "https://youtu.be/.." --creator "Me" --collection "Legit playlist"
```

### Video

With URL only

```zsh
mediadl video "https://youtu.be/..."
```

With creator only:

```zsh
mediadl video "https://youtu.be/.." --creator "HappyPotatoHead"
```

With creator and collection:

```zsh
mediadl video "https://youtu.be/.." --creator "HappyPotatoHead" --collection "Legit playlist"
```

### Batch

> It does not necessarily have to be named `batch.csv`

```zsh
mediadl batch batch.csv --type audio
mediadl batch batch.csv --type video
```

CSV format:

```csv
url,creator,collection
https://youtu.be/example,HappyPotatoHead,Collection of Things
https://youtu.be/example,HappyPotatoHead,@
```

<p align="right"><a href="#readme-top">⬆️</a></p>

## Config

Show config:

```zsh
mediadl config show
```

Edit every config value interactively:

```zsh
mediadl config edit
```

Edit one config value interactively:

```zsh
mediadl config edit audio-format
```

Edit one config value directly:

```zsh
mediadl config set audio-format opus
mediadl config set retries 2
mediadl config set max-parallel-downloads 3
```

Reset config:

```zsh
media config reset
```

### Config keys

- `download-path`
- `audio-format`
- `video-format`
- `video-quality`
- `audio-thumbnail`
- `video-thumbnail`
- `audio-output-template`
- `video-output-template`
- `retries`
- `max-parallel-downloads`

<p align="right"><a href="#readme-top">⬆️</a></p>

## Notes

`mediadl` does not replace yt-dlp. It is a small wrapper that provides a more opinionated workflow for repeated audio/video downloads.

<!-- ROADMAP -->

## Roadmap

- [ ] Refactor code
- [ ] More config options
- [ ] Add prerequisite checks
- [ ] Add [ratatui](https://ratatui.rs/)

See the [open issues](https://github.com/othneildrew/Best-README-Template/issues) for a full list of proposed features (and known issues).

<p align="right"><a href="#readme-top">⬆️</a></p>

<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right"><a href="#readme-top">⬆️</a></p>

<!-- LICENSE -->

## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<p align="right"><a href="#readme-top">⬆️</a></p>

## Disclaimer

This tool is a wrapper for yt-dlp and is intended for personal archival and educational purposes only. The author of `mediadl` does not encourage or condone the use of this software to download copyrighted material in violation of any platform's Terms of Service or local copyright laws. Use this tool responsibly and at your own risk.

This tool is intended to facilitate the exercise of rights under Fair Use for purposes such as criticism, comment, news reporting, teaching, and research.

<!-- CONTACT -->

## Contact

Jimmy Ding - [@instagram](https://www.instagram.com/jmmyd_/) - jimmydingjk@gmail.com

Project Link: [https://github.com/HappyPotatoHead/mediadl](https://github.com/HappyPotatoHead/mediadl)

<p align="right"><a href="#readme-top">⬆️</a></p>

<!-- ACKNOWLEDGMENTS -->

## Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - YouTube downloader
- [ffmpeg](https://ffmpeg.org/) - Audio/video processing

<p align="right"><a href="#readme-top">⬆️</a></p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/HappyPotatoHead/mediadl.svg?style=for-the-badge
[contributors-url]: https://github.com/HappyPotatoHead/mediadl/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/HappyPotatoHead/mediadl.svg?style=for-the-badge
[forks-url]: https://github.com/HappyPotatoHead/mediadl/forks
[stars-shield]: https://img.shields.io/github/stars/HappyPotatoHead/mediadl.svg?style=for-the-badge
[stars-url]: https://github.com/HappyPotatoHead/mediadl/stargazers
[issues-shield]: https://img.shields.io/github/issues/HappyPotatoHead/mediadl.svg?style=for-the-badge
[issues-url]: https://github.com/HappyPotatoHead/mediadl/issues
[license-shield]: https://img.shields.io/github/license/HappyPotatoHead/mediadl.svg?style=for-the-badge
[license-url]: https://github.com/HappyPotatoHead/mediadl/blob/master/LICENSE
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://www.linkedin.com/in/jimmy-ding/
[product-screenshot]: images/screenshot.png
[Rust.com]: https://img.shields.io/badge/rust-0769AD?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://rust-lang.org/
