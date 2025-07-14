# dj-cli

<p align="center">
  <img src="logo.png" alt="dj-cli logo" width="200"/>
</p>

<h2 align="center">Your friendly terminal companion for downloading MP3s from YouTube</h2>

<p align="center">
  <a href="https://crates.io/crates/dj-cli">
    <img src="https://img.shields.io/crates/v/dj-cli?label=crates.io&logo=rust&color=orange" alt="Crates.io version"/>
  </a>
  <a href="https://docs.rs/dj-cli">
    <img src="https://img.shields.io/docsrs/dj-cli?label=docs.rs" alt="docs.rs status"/>
  </a>
  <a href="https://github.com/henryoman/dj-cli/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/henryoman/dj-cli/ci.yml?branch=main&label=CI&logo=github" alt="Build Status"/>
  </a>
  <a href="https://github.com/henryoman/dj-cli/stargazers">
    <img src="https://img.shields.io/github/stars/henryoman/dj-cli?style=social" alt="GitHub stars"/>
  </a>
  <a href="./LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/>
  </a>
</p>

---

## ✨ Features

- **Lightning-fast downloads** using the battle-tested [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) backend
- **Beautiful terminal UI** powered by [`ratatui`](https://ratatui.rs)
- **Async from top to bottom** — built with `tokio` and `futures`
- **Cross-platform**: macOS, Linux, Windows (via WSL)
- **Open Source & MIT-licensed**

---

## 🚀 Quick Start

```bash
# Install the crate from crates.io (recommended)
cargo install dj-cli

# …or run directly from source
git clone https://github.com/henryoman/dj-cli.git
cd dj-cli
cargo run --release
```

Once running, paste any YouTube URL into the prompt and let **dj-cli** fetch and convert the audio to a high-quality MP3.

---

## 🛠️ Building From Source

This project targets the **stable** Rust toolchain (≥ 1.78). If you don’t have Rust yet, install it with [`rustup`](https://www.rust-lang.org/tools/install):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Dependencies

- `ffmpeg` — required by `yt-dlp` for audio extraction & conversion.
  - macOS (Homebrew): `brew install ffmpeg`
  - Debian/Ubuntu: `sudo apt-get install ffmpeg`

---

## 📖 Usage

```bash
# Basic usage – interactive TUI
$ dj-cli

# Download directly without the TUI
dj-cli https://www.youtube.com/watch?v=dQw4w9WgXcQ
```

Output files are saved in the current working directory by default.

---

## 🤝 Contributing

Pull requests are welcome! If you have ideas for new features, feel free to open an issue first to discuss what you’d like to add.

1. Fork the repo and create your branch: `git checkout -b feature/awesome`  
2. Commit your changes: `git commit -m 'Add some awesome feature'`  
3. Push to the branch: `git push origin feature/awesome`  
4. Open a Pull Request.

> **Tip:** Run `cargo clippy --all-targets --all-features -- -D warnings` before pushing to keep the codebase tidy.

---

## 📜 License

This project is licensed under the MIT license — see the [LICENSE](./LICENSE) file for details.
