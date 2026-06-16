use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Extend existing users table — one column at a time (SQLite limitation)
        for alter in [
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(string(Users::PasswordHash).not_null().default(""))
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(string_null(Users::Email))
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(string(Users::Privs).not_null().default("{}"))
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(integer(Users::ScrobblingEnabled).not_null().default(1))
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(integer(Users::MaxBitRate).not_null().default(0))
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(integer(Users::IsEnabled).not_null().default(1))
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .add_column_if_not_exists(big_integer(Users::UpdatedAt).not_null().default(0))
                .to_owned(),
        ] {
            manager.alter_table(alter).await?;
        }

        // libraries
        manager
            .create_table(
                Table::create()
                    .table(Libraries::Table)
                    .if_not_exists()
                    .col(pk_auto(Libraries::Id))
                    .col(string(Libraries::Name).not_null())
                    .col(string(Libraries::Path).unique_key().not_null())
                    .col(integer(Libraries::IsEnabled).not_null().default(1))
                    .col(integer(Libraries::WatchEnabled).not_null().default(0))
                    .col(big_integer(Libraries::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        // user_libraries
        manager
            .create_table(
                Table::create()
                    .table(UserLibraries::Table)
                    .if_not_exists()
                    .col(integer(UserLibraries::UserId).not_null())
                    .col(integer(UserLibraries::LibraryId).not_null())
                    .col(integer(UserLibraries::IsEnabled).not_null().default(1))
                    .primary_key(
                        Index::create()
                            .col(UserLibraries::UserId)
                            .col(UserLibraries::LibraryId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_libraries_user")
                            .from(UserLibraries::Table, UserLibraries::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_libraries_library")
                            .from(UserLibraries::Table, UserLibraries::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // artists
        manager
            .create_table(
                Table::create()
                    .table(Artists::Table)
                    .if_not_exists()
                    .col(pk_auto(Artists::Id))
                    .col(string(Artists::Name).not_null())
                    .col(string_null(Artists::SortName))
                    .col(integer(Artists::LibraryId).not_null())
                    .col(big_integer(Artists::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_artists_library")
                            .from(Artists::Table, Artists::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_artists_name")
                    .table(Artists::Table)
                    .col(Artists::Name)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_artists_sort_name")
                    .table(Artists::Table)
                    .col(Artists::SortName)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_artists_library_id")
                    .table(Artists::Table)
                    .col(Artists::LibraryId)
                    .to_owned(),
            )
            .await?;

        // albums
        manager
            .create_table(
                Table::create()
                    .table(Albums::Table)
                    .if_not_exists()
                    .col(pk_auto(Albums::Id))
                    .col(string(Albums::Name).not_null())
                    .col(integer(Albums::ArtistId).not_null())
                    .col(integer_null(Albums::Year))
                    .col(integer(Albums::LibraryId).not_null())
                    .col(big_integer(Albums::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_albums_artist")
                            .from(Albums::Table, Albums::ArtistId)
                            .to(Artists::Table, Artists::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_albums_library")
                            .from(Albums::Table, Albums::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_albums_name")
                    .table(Albums::Table)
                    .col(Albums::Name)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_albums_artist_id")
                    .table(Albums::Table)
                    .col(Albums::ArtistId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_albums_year")
                    .table(Albums::Table)
                    .col(Albums::Year)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_albums_library_id")
                    .table(Albums::Table)
                    .col(Albums::LibraryId)
                    .to_owned(),
            )
            .await?;

        // songs
        manager
            .create_table(
                Table::create()
                    .table(Songs::Table)
                    .if_not_exists()
                    .col(pk_auto(Songs::Id))
                    .col(string(Songs::Title).not_null())
                    .col(integer(Songs::ArtistId).not_null())
                    .col(integer_null(Songs::AlbumId))
                    .col(integer_null(Songs::TrackNumber))
                    .col(integer_null(Songs::DiscNumber))
                    .col(integer(Songs::DurationSecs).not_null())
                    .col(integer_null(Songs::BitRate))
                    .col(big_integer_null(Songs::SizeBytes))
                    .col(string_null(Songs::FileFormat))
                    .col(string_null(Songs::ContentType))
                    .col(integer_null(Songs::Year))
                    .col(string(Songs::FilePath).unique_key().not_null())
                    .col(integer(Songs::HasCoverArt).not_null().default(0))
                    .col(integer(Songs::LibraryId).not_null())
                    .col(big_integer(Songs::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_songs_artist")
                            .from(Songs::Table, Songs::ArtistId)
                            .to(Artists::Table, Artists::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_songs_album")
                            .from(Songs::Table, Songs::AlbumId)
                            .to(Albums::Table, Albums::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_songs_library")
                            .from(Songs::Table, Songs::LibraryId)
                            .to(Libraries::Table, Libraries::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_songs_title")
                    .table(Songs::Table)
                    .col(Songs::Title)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_songs_artist_id")
                    .table(Songs::Table)
                    .col(Songs::ArtistId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_songs_album_id")
                    .table(Songs::Table)
                    .col(Songs::AlbumId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_songs_year")
                    .table(Songs::Table)
                    .col(Songs::Year)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_songs_library_id")
                    .table(Songs::Table)
                    .col(Songs::LibraryId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_songs_created_at")
                    .table(Songs::Table)
                    .col(Songs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // playlists
        manager
            .create_table(
                Table::create()
                    .table(Playlists::Table)
                    .if_not_exists()
                    .col(pk_auto(Playlists::Id))
                    .col(string(Playlists::Name).not_null())
                    .col(integer(Playlists::OwnerId).not_null())
                    .col(string_null(Playlists::Comment))
                    .col(integer(Playlists::IsPublic).not_null().default(0))
                    .col(big_integer(Playlists::CreatedAt).not_null())
                    .col(big_integer(Playlists::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlists_owner")
                            .from(Playlists::Table, Playlists::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_playlists_owner_id")
                    .table(Playlists::Table)
                    .col(Playlists::OwnerId)
                    .to_owned(),
            )
            .await?;

        // playlist_songs
        manager
            .create_table(
                Table::create()
                    .table(PlaylistSongs::Table)
                    .if_not_exists()
                    .col(integer(PlaylistSongs::PlaylistId).not_null())
                    .col(integer(PlaylistSongs::SongId).not_null())
                    .col(integer(PlaylistSongs::Position).not_null())
                    .col(big_integer(PlaylistSongs::AddedAt).not_null())
                    .primary_key(
                        Index::create()
                            .col(PlaylistSongs::PlaylistId)
                            .col(PlaylistSongs::SongId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_songs_playlist")
                            .from(PlaylistSongs::Table, PlaylistSongs::PlaylistId)
                            .to(Playlists::Table, Playlists::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_playlist_songs_song")
                            .from(PlaylistSongs::Table, PlaylistSongs::SongId)
                            .to(Songs::Table, Songs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // stars
        manager
            .create_table(
                Table::create()
                    .table(Stars::Table)
                    .if_not_exists()
                    .col(integer(Stars::UserId).not_null())
                    .col(string(Stars::ItemType).not_null())
                    .col(integer(Stars::ItemId).not_null())
                    .col(big_integer(Stars::StarredAt).not_null())
                    .primary_key(
                        Index::create()
                            .col(Stars::UserId)
                            .col(Stars::ItemType)
                            .col(Stars::ItemId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_stars_user")
                            .from(Stars::Table, Stars::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_stars_user_id")
                    .table(Stars::Table)
                    .col(Stars::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_stars_item")
                    .table(Stars::Table)
                    .col(Stars::ItemType)
                    .col(Stars::ItemId)
                    .to_owned(),
            )
            .await?;

        // ratings
        manager
            .create_table(
                Table::create()
                    .table(Ratings::Table)
                    .if_not_exists()
                    .col(integer(Ratings::UserId).not_null())
                    .col(string(Ratings::ItemType).not_null())
                    .col(integer(Ratings::ItemId).not_null())
                    .col(integer(Ratings::Rating).not_null())
                    .col(big_integer(Ratings::RatedAt).not_null())
                    .primary_key(
                        Index::create()
                            .col(Ratings::UserId)
                            .col(Ratings::ItemType)
                            .col(Ratings::ItemId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ratings_user")
                            .from(Ratings::Table, Ratings::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // scrobbles
        manager
            .create_table(
                Table::create()
                    .table(Scrobbles::Table)
                    .if_not_exists()
                    .col(pk_auto(Scrobbles::Id))
                    .col(integer(Scrobbles::UserId).not_null())
                    .col(integer(Scrobbles::SongId).not_null())
                    .col(integer(Scrobbles::Submission).not_null().default(1))
                    .col(big_integer(Scrobbles::PlayedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_scrobbles_user")
                            .from(Scrobbles::Table, Scrobbles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_scrobbles_song")
                            .from(Scrobbles::Table, Scrobbles::SongId)
                            .to(Songs::Table, Songs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_scrobbles_user_song_time")
                    .table(Scrobbles::Table)
                    .col(Scrobbles::UserId)
                    .col(Scrobbles::SongId)
                    .col(Scrobbles::PlayedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_scrobbles_user_played")
                    .table(Scrobbles::Table)
                    .col(Scrobbles::UserId)
                    .col(Scrobbles::PlayedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_scrobbles_song_id")
                    .table(Scrobbles::Table)
                    .col(Scrobbles::SongId)
                    .to_owned(),
            )
            .await?;

        // bookmarks
        manager
            .create_table(
                Table::create()
                    .table(Bookmarks::Table)
                    .if_not_exists()
                    .col(pk_auto(Bookmarks::Id))
                    .col(integer(Bookmarks::UserId).not_null())
                    .col(integer(Bookmarks::SongId).not_null())
                    .col(integer(Bookmarks::PositionMs).not_null())
                    .col(string_null(Bookmarks::DeviceId))
                    .col(big_integer(Bookmarks::UpdatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bookmarks_user")
                            .from(Bookmarks::Table, Bookmarks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bookmarks_song")
                            .from(Bookmarks::Table, Bookmarks::SongId)
                            .to(Songs::Table, Songs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // shares
        manager
            .create_table(
                Table::create()
                    .table(Shares::Table)
                    .if_not_exists()
                    .col(pk_auto(Shares::Id))
                    .col(integer(Shares::OwnerId).not_null())
                    .col(string(Shares::ItemType).not_null())
                    .col(integer(Shares::ItemId).not_null())
                    .col(string_null(Shares::Description))
                    .col(string(Shares::Token).unique_key().not_null())
                    .col(integer(Shares::VisitCount).not_null().default(0))
                    .col(big_integer_null(Shares::LastVisitedAt))
                    .col(big_integer_null(Shares::ExpiresAt))
                    .col(big_integer(Shares::CreatedAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_shares_owner")
                            .from(Shares::Table, Shares::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_shares_token")
                    .table(Shares::Table)
                    .col(Shares::Token)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_shares_owner_id")
                    .table(Shares::Table)
                    .col(Shares::OwnerId)
                    .to_owned(),
            )
            .await?;

        // sessions
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(string(Sessions::Id).primary_key())
                    .col(integer(Sessions::UserId).not_null())
                    .col(string(Sessions::Token).unique_key().not_null())
                    .col(string_null(Sessions::DeviceInfo))
                    .col(big_integer(Sessions::CreatedAt).not_null())
                    .col(big_integer(Sessions::LastUsedAt).not_null())
                    .col(big_integer(Sessions::ExpiresAt).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_user")
                            .from(Sessions::Table, Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_sessions_token")
                    .table(Sessions::Table)
                    .col(Sessions::Token)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_sessions_user_id")
                    .table(Sessions::Table)
                    .col(Sessions::UserId)
                    .to_owned(),
            )
            .await?;

        // password_resets
        manager
            .create_table(
                Table::create()
                    .table(PasswordResets::Table)
                    .if_not_exists()
                    .col(pk_auto(PasswordResets::Id))
                    .col(string(PasswordResets::Email).not_null())
                    .col(string(PasswordResets::Code).not_null())
                    .col(big_integer(PasswordResets::ExpiresAt).not_null())
                    .col(integer(PasswordResets::Used).not_null().default(0))
                    .col(big_integer(PasswordResets::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_password_resets_email")
                    .table(PasswordResets::Table)
                    .col(PasswordResets::Email)
                    .to_owned(),
            )
            .await?;

        // scan_tasks
        manager
            .create_table(
                Table::create()
                    .table(ScanTasks::Table)
                    .if_not_exists()
                    .col(string(ScanTasks::Id).primary_key())
                    .col(string(ScanTasks::Status).not_null())
                    .col(integer(ScanTasks::FilesScanned).not_null().default(0))
                    .col(integer(ScanTasks::FilesTotal).not_null().default(0))
                    .col(big_integer(ScanTasks::StartedAt).not_null())
                    .col(big_integer_null(ScanTasks::FinishedAt))
                    .col(string_null(ScanTasks::Error))
                    .to_owned(),
            )
            .await?;

        // lyrics
        manager
            .create_table(
                Table::create()
                    .table(Lyrics::Table)
                    .if_not_exists()
                    .col(integer(Lyrics::SongId).primary_key())
                    .col(string(Lyrics::Type).not_null())
                    .col(string(Lyrics::Content).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_lyrics_song")
                            .from(Lyrics::Table, Lyrics::SongId)
                            .to(Songs::Table, Songs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // cover_art
        manager
            .create_table(
                Table::create()
                    .table(CoverArt::Table)
                    .if_not_exists()
                    .col(pk_auto(CoverArt::Id))
                    .col(string(CoverArt::ItemType).not_null())
                    .col(integer(CoverArt::ItemId).not_null())
                    .col(string(CoverArt::MimeType).not_null())
                    .col(integer_null(CoverArt::Width))
                    .col(integer_null(CoverArt::Height))
                    .col(string_null(CoverArt::FilePath))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cover_art_item")
                    .table(CoverArt::Table)
                    .col(CoverArt::ItemType)
                    .col(CoverArt::ItemId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CoverArt::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Lyrics::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ScanTasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PasswordResets::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Shares::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Bookmarks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scrobbles::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Ratings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Stars::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PlaylistSongs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Playlists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Songs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Albums::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Artists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserLibraries::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Libraries::Table).to_owned())
            .await?;

        // Remove columns from users — one at a time (SQLite limitation)
        for alter in [
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::PasswordHash)
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::Email)
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::Privs)
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::ScrobblingEnabled)
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::MaxBitRate)
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::IsEnabled)
                .to_owned(),
            Table::alter()
                .table(Users::Table)
                .drop_column(Users::UpdatedAt)
                .to_owned(),
        ] {
            manager.alter_table(alter).await?;
        }

        Ok(())
    }
}

// Table identifiers for the original users table (extended)
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    CreatedAt,
    PasswordHash,
    Email,
    Privs,
    ScrobblingEnabled,
    MaxBitRate,
    IsEnabled,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Libraries {
    Table,
    Id,
    Name,
    Path,
    IsEnabled,
    WatchEnabled,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserLibraries {
    Table,
    UserId,
    LibraryId,
    IsEnabled,
}

#[derive(DeriveIden)]
enum Artists {
    Table,
    Id,
    Name,
    SortName,
    LibraryId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Albums {
    Table,
    Id,
    Name,
    ArtistId,
    Year,
    LibraryId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Songs {
    Table,
    Id,
    Title,
    ArtistId,
    AlbumId,
    TrackNumber,
    DiscNumber,
    DurationSecs,
    BitRate,
    SizeBytes,
    FileFormat,
    ContentType,
    Year,
    FilePath,
    HasCoverArt,
    LibraryId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Playlists {
    Table,
    Id,
    Name,
    OwnerId,
    Comment,
    IsPublic,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum PlaylistSongs {
    Table,
    PlaylistId,
    SongId,
    Position,
    AddedAt,
}

#[derive(DeriveIden)]
enum Stars {
    Table,
    UserId,
    ItemType,
    ItemId,
    StarredAt,
}

#[derive(DeriveIden)]
enum Ratings {
    Table,
    UserId,
    ItemType,
    ItemId,
    Rating,
    RatedAt,
}

#[derive(DeriveIden)]
enum Scrobbles {
    Table,
    Id,
    UserId,
    SongId,
    Submission,
    PlayedAt,
}

#[derive(DeriveIden)]
enum Bookmarks {
    Table,
    Id,
    UserId,
    SongId,
    PositionMs,
    DeviceId,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Shares {
    Table,
    Id,
    OwnerId,
    ItemType,
    ItemId,
    Description,
    Token,
    VisitCount,
    LastVisitedAt,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    UserId,
    Token,
    DeviceInfo,
    CreatedAt,
    LastUsedAt,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum PasswordResets {
    Table,
    Id,
    Email,
    Code,
    ExpiresAt,
    Used,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ScanTasks {
    Table,
    Id,
    Status,
    FilesScanned,
    FilesTotal,
    StartedAt,
    FinishedAt,
    Error,
}

#[derive(DeriveIden)]
enum Lyrics {
    Table,
    SongId,
    Type,
    Content,
}

#[derive(DeriveIden)]
enum CoverArt {
    Table,
    Id,
    ItemType,
    ItemId,
    MimeType,
    Width,
    Height,
    FilePath,
}
