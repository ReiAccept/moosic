use axum::{middleware, routing::{get, post}, Router};
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers;
use crate::middleware::auth::auth_middleware;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let frontend_dir = state.config.frontend.path.clone();

    // Public API routes — no authentication required
    let public = Router::new()
        .route("/api/health", get(handlers::health))
        .route("/api/user/login", post(handlers::login))
        .route("/api/user/password/reset/request", post(handlers::password_reset_request))
        .route("/api/user/password/reset/confirm", post(handlers::password_reset_confirm))
        .route("/api/share/{token}", get(handlers::get_share));

    // Protected API routes — require valid Bearer token
    let protected = Router::new()
        // User self-service
        .route("/api/user/logout", get(handlers::logout))
        .route("/api/user/info", get(handlers::info))
        .route("/api/user/edit", post(handlers::edit))
        .route("/api/user/password/edit", post(handlers::password_edit))
        .route("/api/user/token/refresh", post(handlers::token_refresh))
        .route("/api/user/sessions", post(handlers::sessions))
        .route("/api/user/session/revoke", post(handlers::session_revoke))
        .route("/api/user/delete", post(handlers::delete_account))
        // Music
        .route("/api/music/info", post(handlers::music_info))
        .route("/api/music/stream", get(handlers::music_stream))
        .route("/api/music/download", get(handlers::music_download))
        .route("/api/music/cover", get(handlers::music_cover))
        .route("/api/music/lyrics", get(handlers::music_lyrics))
        .route("/api/music/rand", post(handlers::music_rand))
        .route("/api/music/playing", post(handlers::music_playing))
        .route("/api/music/list", post(handlers::music_list))
        // Album
        .route("/api/album/info", post(handlers::album_info))
        .route("/api/album/list", post(handlers::album_list))
        .route("/api/album/cover", get(handlers::album_cover))
        // Artist
        .route("/api/artist/info", post(handlers::artist_info))
        .route("/api/artist/list", post(handlers::artist_list))
        .route("/api/artist/cover", get(handlers::artist_cover))
        .route("/api/artist/songs", post(handlers::artist_songs))
        // Playlist
        .route("/api/playlist/list", post(handlers::playlist_list))
        .route("/api/playlist/info", post(handlers::playlist_info))
        .route("/api/playlist/create", post(handlers::playlist_create))
        .route("/api/playlist/update", post(handlers::playlist_update))
        .route("/api/playlist/del", post(handlers::playlist_del))
        .route("/api/playlist/music/add", post(handlers::playlist_add_songs))
        .route("/api/playlist/music/remove", post(handlers::playlist_remove_songs))
        .route("/api/playlist/music/reorder", post(handlers::playlist_reorder))
        .route("/api/playlist/cover", get(handlers::playlist_cover))
        // Library
        .route("/api/library/list", post(handlers::library_list))
        .route("/api/library/enable", post(handlers::library_enable))
        .route("/api/library/disable", post(handlers::library_disable))
        .route("/api/library/rescan", post(handlers::library_rescan))
        // Search
        .route("/api/search", post(handlers::search))
        .route("/api/search/suggest", post(handlers::search_suggest))
        // Bookmark
        .route("/api/bookmark/list", post(handlers::bookmark_list))
        .route("/api/bookmark/get", post(handlers::bookmark_get))
        .route("/api/bookmark/create", post(handlers::bookmark_create))
        .route("/api/bookmark/delete", post(handlers::bookmark_delete))
        // Share (authenticated endpoints)
        .route("/api/share/list", post(handlers::share_list))
        .route("/api/share/create", post(handlers::share_create))
        .route("/api/share/update", post(handlers::share_update))
        .route("/api/share/delete", post(handlers::share_delete))
        .route("/api/share/delete/batch", post(handlers::share_delete_batch))
        // Annotation
        .route("/api/annotation/star", post(handlers::toggle_star))
        .route("/api/annotation/rate", post(handlers::set_rating))
        .route("/api/annotation/rate/clear", post(handlers::clear_rating))
        .route("/api/annotation/scrobble", post(handlers::scrobble))
        .route("/api/annotation/starred", post(handlers::starred_list))
        .route("/api/annotation/rated", post(handlers::rated_list))
        .route("/api/annotation/history", post(handlers::play_history))
        // Admin — User
        .route("/api/admin/user/add", post(handlers::admin_user::add_user))
        .route("/api/admin/user/del", post(handlers::admin_user::del_user))
        .route("/api/admin/user/password/edit", post(handlers::admin_user::admin_password_edit))
        .route("/api/admin/user/list", post(handlers::admin_user::list_users))
        .route("/api/admin/user/info", post(handlers::admin_user::user_info))
        .route("/api/admin/user/priv/edit", post(handlers::admin_user::edit_privs))
        .route("/api/admin/user/edit", post(handlers::admin_user::edit_user))
        .route("/api/admin/user/enable", post(handlers::admin_user::enable_user))
        .route("/api/admin/user/disable", post(handlers::admin_user::disable_user))
        // Admin — Library
        .route("/api/admin/library/add", post(handlers::admin_library::add_library))
        .route("/api/admin/library/del", post(handlers::admin_library::del_library))
        .route("/api/admin/library/update", post(handlers::admin_library::update_library))
        .route("/api/admin/library/notify/enable", post(handlers::admin_library::enable_notify))
        .route("/api/admin/library/notify/disable", post(handlers::admin_library::disable_notify))
        .route("/api/admin/library/scan", post(handlers::admin_library::start_scan))
        .route("/api/admin/library/scan/all", post(handlers::admin_library::start_full_scan))
        .route("/api/admin/library/scan/status", post(handlers::admin_library::scan_status))
        .route("/api/admin/library/scan/cancel", post(handlers::admin_library::cancel_scan))
        // Admin — Server
        .route("/api/admin/server/status", get(handlers::server_status))
        // Apply auth middleware
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Assemble: API routes first, then fall back to static frontend files.
    // For any unmatched path (SPA client-side routes), serve index.html.
    Router::new()
        .merge(public)
        .merge(protected)
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .fallback(ServeFile::new(format!("{frontend_dir}/index.html")))
        )
        .with_state(state)
}
