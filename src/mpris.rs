use crate::player::PlayerState;
use std::sync::Arc;
use std::sync::Mutex;

use mpris_server::{
    LoopStatus, Metadata, PlaybackRate, PlaybackStatus, PlayerInterface, RootInterface, Time,
    TrackId, Volume,
    zbus::{Result, fdo},
};

// #[derive(Debug)]
// pub enum MprisState {
//     Play,
//     Pause,
//     PlayPause,
//     Next,
//     Previous,
//     Shuffle(bool),
//     Loop,
//     Volume(f64),
//     Seek(i64),
//     Stop,
//     Position(i64),
// }

pub struct MprisHandler {
    pub state: Arc<Mutex<PlayerState>>,
}

impl RootInterface for MprisHandler {
    async fn raise(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> fdo::Result<()> {
        Ok(())
    }

    async fn can_quit(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_fullscreen(&self, _: bool) -> Result<()> {
        Ok(())
    }

    async fn can_set_fullscreen(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn can_raise(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn has_track_list(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn identity(&self) -> fdo::Result<String> {
        Ok("SanctumPlayer".to_string())
    }

    async fn desktop_entry(&self) -> fdo::Result<String> {
        Ok("Sanctum.Player".to_string())
    }

    async fn supported_uri_schemes(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }

    async fn supported_mime_types(&self) -> fdo::Result<Vec<String>> {
        Ok(vec![])
    }
}

impl PlayerInterface for MprisHandler {
    async fn next(&self) -> fdo::Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.skip = true;
        }

        Ok(())
    }

    async fn previous(&self) -> fdo::Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.previous = true;
        }
        Ok(())
    }

    async fn pause(&self) -> fdo::Result<()> {
        println!("Triggered: Pause");
        if let Ok(mut state) = self.state.lock() {
            state.pause = true;
        }

        Ok(())
    }

    async fn play_pause(&self) -> fdo::Result<()> {
        println!("Triggered: Play Pause");
        if let Ok(mut state) = self.state.lock() {
            state.play_pause = true;
        }

        Ok(())
    }

    async fn stop(&self) -> fdo::Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.stop = true;
        }

        Ok(())
    }

    async fn play(&self) -> fdo::Result<()> {
        println!("Triggered: Play");
        if let Ok(mut state) = self.state.lock() {
            state.play = true;
        }
        Ok(())
    }

    async fn shuffle(&self) -> fdo::Result<bool> {
        Ok(false)
    }

    async fn set_shuffle(&self, shuffle: bool) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.shuffle = shuffle;
        }

        Ok(())
    }

    async fn position(&self) -> fdo::Result<Time> {
        let state = self.state.lock().unwrap();
        Ok(Time::from_secs(state.player_pos as i64))
    }

    async fn set_position(&self, _track_id: TrackId, position: Time) -> fdo::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.mpris_pos = position.as_secs() as u64;
        Ok(())
    }

    async fn open_uri(&self, uri: String) -> fdo::Result<()> {
        println!("OpenUri({uri})");
        Ok(())
    }

    async fn playback_status(&self) -> fdo::Result<PlaybackStatus> {
        let state = self.state.lock().unwrap();
        Ok(state.status)
    }

    async fn loop_status(&self) -> fdo::Result<LoopStatus> {
        let state = self.state.lock().unwrap();

        let loop_status = state.repeat;
        let loop_state: LoopStatus = if loop_status {
            LoopStatus::Track
        } else {
            LoopStatus::None
        };

        Ok(loop_state)
    }

    async fn set_loop_status(&self, loop_status: LoopStatus) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            match loop_status {
                LoopStatus::Track => state.repeat = true,
                LoopStatus::None => state.repeat = false,
                _ => state.repeat = false,
            }
        }
        Ok(())
    }

    async fn rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(PlaybackRate::default())
    }

    async fn set_rate(&self, rate: PlaybackRate) -> Result<()> {
        println!("No support for set ({rate})");
        Ok(())
    }

    async fn metadata(&self) -> fdo::Result<Metadata> {
        let state = self.state.lock().unwrap();
        let metadata = state.metadata.clone();
        Ok(metadata)
    }

    async fn volume(&self) -> fdo::Result<Volume> {
        let state = self.state.lock().unwrap();
        let volume = state.volume as f64;
        Ok(volume)
    }

    async fn set_volume(&self, volume: Volume) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.volume = volume as u64;
        }

        Ok(())
    }

    async fn can_go_next(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_go_previous(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_play(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_pause(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn seek(&self, offset: Time) -> fdo::Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.mpris_pos = offset.as_secs() as u64;
        }

        Ok(())
    }

    async fn can_seek(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn can_control(&self) -> fdo::Result<bool> {
        Ok(true)
    }

    async fn minimum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(PlaybackRate::default())
    }

    async fn maximum_rate(&self) -> fdo::Result<PlaybackRate> {
        Ok(PlaybackRate::default())
    }
}
