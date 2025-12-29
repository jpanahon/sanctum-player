# Sanctum Player
A local music player that supports .mp3, .flac, and .m4a written in Rust

![showcase](assets/showcase.png "Demo")

# Features
- Search
- Album Art
- Shuffle

## TODO
- MPRIS Support
- Add onboarding page
- Import songs while application is open
- Make proper queue system
- Filter songs btased on tags

# Libraries
- [egui](https://github.com/emilk/egui) (User Interface)
- [egui_extras](https://github.com/emilk/egui/blob/main/crates/egui_extras/README.md) (egui Image Support)
- [rodio](https://github.com/RustAudio/rodio) (Audio Playback)
- [lofty](https://github.com/Serial-ATA/lofty-rs) (Audio Metadata)
- [image](https://github.com/image-rs/image) (Image Loading)
- [fuzzy_matcher](https://github.com/skim-rs/fuzzy-matcher) (Fuzzy Searching)
