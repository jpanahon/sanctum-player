use crate::player::Song;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub enum Sort {
    Track { reverse: bool },
    Title { reverse: bool },
    Artist { reverse: bool },
    Album { reverse: bool },
    Date { reverse: bool },
    Time { reverse: bool },
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Playlist {
    pub name: String,
    pub path: String,
    pub sort_order: Sort,
}

pub fn sort_songs(playlist: Playlist, song_view: &mut Vec<usize>, songs: &Vec<Song>) {
    let sort = playlist.sort_order();
    song_view.sort_by(|&view, &song| {
        let (order, reverse) = match sort {
            Sort::Track { reverse } => (view.cmp(&song), reverse),
            Sort::Title { reverse } => (
                songs[view]
                    .title
                    .clone()
                    .to_lowercase()
                    .cmp(&songs[song].title.clone().to_lowercase()),
                reverse,
            ),
            Sort::Artist { reverse } => (
                songs[view]
                    .artist
                    .clone()
                    .to_lowercase()
                    .cmp(&songs[song].artist.clone().to_lowercase()),
                reverse,
            ),
            Sort::Album { reverse } => (
                songs[view]
                    .album
                    .clone()
                    .to_lowercase()
                    .cmp(&songs[song].album.clone().to_lowercase()),
                reverse,
            ),
            Sort::Date { reverse } => (songs[view].created.cmp(&songs[song].created), reverse),
            Sort::Time { reverse } => (songs[view].duration.cmp(&songs[song].duration), reverse),
        };

        if reverse { order.reverse() } else { order }
    });
}

impl Playlist {
    pub fn set_sort(&mut self, sort_order: Sort) {
        self.sort_order = sort_order
    }

    pub fn sort_order(self) -> Sort {
        self.sort_order
    }
}
