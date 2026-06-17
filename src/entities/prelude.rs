pub use super::albums::{ActiveModel as AlbumActiveModel, Entity as AlbumEntity};
pub use super::artists::{ActiveModel as ArtistActiveModel, Entity as ArtistEntity};
pub use super::bookmarks::{ActiveModel as BookmarkActiveModel, Entity as BookmarkEntity};
pub use super::cover_art::{ActiveModel as CoverArtActiveModel, Entity as CoverArtEntity};
pub use super::libraries::{ActiveModel as LibraryActiveModel, Entity as LibraryEntity};
pub use super::lyrics::{ActiveModel as LyricActiveModel, Entity as LyricEntity};
pub use super::playlist_songs::{ActiveModel as PlaylistSongActiveModel, Entity as PlaylistSongEntity};
pub use super::playlists::{ActiveModel as PlaylistActiveModel, Entity as PlaylistEntity};
pub use super::ratings::{ActiveModel as RatingActiveModel, Entity as RatingEntity};
pub use super::scan_tasks::{ActiveModel as ScanTaskActiveModel, Entity as ScanTaskEntity};
pub use super::scrobbles::{ActiveModel as ScrobbleActiveModel, Entity as ScrobbleEntity};
pub use super::sessions::{ActiveModel as SessionActiveModel, Entity as SessionEntity};
pub use super::shares::{ActiveModel as ShareActiveModel, Entity as ShareEntity};
pub use super::songs::{ActiveModel as SongActiveModel, Entity as SongEntity};
pub use super::stars::{ActiveModel as StarActiveModel, Entity as StarEntity};
pub use super::user_libraries::{ActiveModel as UserLibraryActiveModel, Entity as UserLibraryEntity};
pub use super::users::{ActiveModel as UserActiveModel, Entity as UserEntity};

pub use sea_orm::entity::prelude::*;
