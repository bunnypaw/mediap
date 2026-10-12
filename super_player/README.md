# Super Player (Native Pure Rust + libmpv)

High-performance native desktop & mobile media player written in pure Rust with direct C API bindings to `libmpv`. Inspired by the [VLC Community Redesign](https://www.figma.com/community/file/1691465999134370980).

## 🚀 Key Architectural Highlights
- **100% Pure Native Rust**: No Electron, no Tauri, no web views. Ultra-low memory footprint and zero JavaScript runtime overhead.
- **Hardware-Accelerated Decoding**: Direct `libmpv` integration supporting:
  - **NVDEC** on NVIDIA GPUs via CUDA / Direct3D11
  - **VAAPI** on Linux Intel / AMD GPUs
  - **MediaCodec** on Android / Googlebook OS (ARM64)
- **Zero Local Compilation**: Fully automated cloud compilation via GitHub Actions matrix workflows for Linux x64, Windows x64, and Android ARM64.
- **VLC Community Redesign UI**: Floating player controls dock, dual-timestamp timeline scrubber, volume popover, dark/light theme toggle, and playlist drawer.

## 📁 Repository Structure
```
super_player/
├── Cargo.toml               # Native dependencies (libmpv2, iced, tokio, reqwest, rfd)
├── README.md
└── src/
    ├── main.rs              # App entry point, event loop, and file dialog handlers
    ├── player.rs            # Safe libmpv wrapper & hardware acceleration engine
    ├── theme.rs             # Figma VLC Redesign palette tokens (Light & Dark)
    └── ui.rs                # Responsive UI, floating controls, playlist drawer
.github/
└── workflows/
    └── build.yml            # Multi-platform cloud compilation matrix
web/
└── index.html               # Lightweight release hub & Gemini playlist assistant
```

## ☁ Cloud Compilation via GitHub Actions
To build native binaries without installing Rust or compilers locally:
1. Push this repository to GitHub: `git push origin main`
2. Go to the **Actions** tab in your repository.
3. The `Build & Release Super Player Matrix` workflow automatically triggers:
   - Compiles `super_player-linux-x86_64` on Ubuntu runners with `libmpv-dev`
   - Compiles `super_player-windows-x86_64.exe` bundled with `mpv-2.dll`
   - Compiles `super_player-android-aarch64` with Android NDK and `cargo-ndk`
4. Download the compiled native binaries directly from the **Artifacts** or **Releases** section.
