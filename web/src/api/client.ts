const BASE = '/api';

// Get stored token
function getToken(): string | null {
  return localStorage.getItem('moosic_token');
}

// Generic fetch wrapper
async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${BASE}${path}`, { ...options, headers });

  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    const msg = body?.error?.message || res.statusText;
    throw new Error(msg);
  }

  // Handle 204 No Content
  if (res.status === 204) return undefined as T;

  return res.json();
}

// Auth
export async function login(username: string, password: string) {
  const data = await request<{ token: string; user: UserInfo }>('/user/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
  localStorage.setItem('moosic_token', data.token);
  localStorage.setItem('moosic_user', JSON.stringify(data.user));
  return data;
}

export function logout(): void {
  localStorage.removeItem('moosic_token');
  localStorage.removeItem('moosic_user');
}

export function getSavedUser(): UserInfo | null {
  const raw = localStorage.getItem('moosic_user');
  return raw ? JSON.parse(raw) : null;
}

// Health
export const health = () => request<{ status: string; version: string }>('/health');

// User
export const userInfo = () => request<UserInfo>('/user/info');
export const editUser = (data: EditUserRequest) => request<UserInfo>('/user/edit', { method: 'POST', body: JSON.stringify(data) });
export const changePassword = (data: { old_password: string; new_password: string }) => request('/user/password/edit', { method: 'POST', body: JSON.stringify(data) });
export const listSessions = () => request<{ sessions: SessionInfo[] }>('/user/sessions', { method: 'POST', body: '{}' });
export const revokeSession = (session_id: string) => request('/user/session/revoke', { method: 'POST', body: JSON.stringify({ session_id }) });
export const deleteAccount = (password: string) => request('/user/delete', { method: 'POST', body: JSON.stringify({ password }) });
export const tokenRefresh = () => request<{ token: string }>('/user/token/refresh', { method: 'POST', body: '{}' });

// Music
export const musicInfo = (id: number) => request<SongDetail>('/music/info', { method: 'POST', body: JSON.stringify({ id }) });
export const musicStreamUrl = (id: number) => `${BASE}/music/stream?id=${id}`;
export const musicDownloadUrl = (id: number) => `${BASE}/music/download?id=${id}`;
export const musicCoverUrl = (id: number, size?: number) => `${BASE}/music/cover?id=${id}${size ? `&size=${size}` : ''}`;
export const musicLyrics = (id: number) => request<LyricsData>(`/music/lyrics?id=${id}`);
export const musicRand = (size?: number) => request<{ songs: SongItem[] }>('/music/rand', { method: 'POST', body: JSON.stringify({ size: size || 20 }) });
export const musicPlaying = () => request<{ entries: NowPlayingEntry[] }>('/music/playing', { method: 'POST', body: '{}' });
export const musicList = (params: MusicListParams) => request<{ songs: SongItem[]; total: number }>('/music/list', { method: 'POST', body: JSON.stringify(params) });

// Album
export const albumInfo = (id: number) => request<AlbumDetail>('/album/info', { method: 'POST', body: JSON.stringify({ id }) });
export const albumList = (params: AlbumListParams) => request<{ albums: AlbumItem[]; total: number }>('/album/list', { method: 'POST', body: JSON.stringify(params) });
export const albumCoverUrl = (id: number, size?: number) => `${BASE}/album/cover?id=${id}${size ? `&size=${size}` : ''}`;

// Artist
export const artistInfo = (id: number) => request<ArtistDetail>('/artist/info', { method: 'POST', body: JSON.stringify({ id }) });
export const artistList = (params: ArtistListParams) => request<{ artists: ArtistItem[]; total: number }>('/artist/list', { method: 'POST', body: JSON.stringify(params) });
export const artistCoverUrl = (id: number, size?: number) => `${BASE}/artist/cover?id=${id}${size ? `&size=${size}` : ''}`;
export const artistSongs = (id: number, offset?: number) => request<{ songs: SongItem[]; total: number }>('/artist/songs', { method: 'POST', body: JSON.stringify({ id, offset, limit: 50 }) });

// Playlist
export const playlistList = () => request<{ playlists: PlaylistSummary[] }>('/playlist/list', { method: 'POST', body: '{}' });
export const playlistInfo = (id: number) => request<PlaylistDetail>('/playlist/info', { method: 'POST', body: JSON.stringify({ id }) });
export const playlistCreate = (data: { name: string; comment?: string; is_public?: boolean; song_ids?: number[] }) => request<PlaylistSummary>('/playlist/create', { method: 'POST', body: JSON.stringify(data) });
export const playlistUpdate = (data: { id: number; name?: string; comment?: string | null; is_public?: boolean }) => request('/playlist/update', { method: 'POST', body: JSON.stringify(data) });
export const playlistDelete = (id: number) => request('/playlist/del', { method: 'POST', body: JSON.stringify({ id }) });
export const playlistAddSongs = (id: number, song_ids: number[]) => request('/playlist/music/add', { method: 'POST', body: JSON.stringify({ id, song_ids }) });
export const playlistRemoveSongs = (id: number, song_ids: number[]) => request('/playlist/music/remove', { method: 'POST', body: JSON.stringify({ id, song_ids }) });
export const playlistCoverUrl = (id: number, size?: number) => `${BASE}/playlist/cover?id=${id}${size ? `&size=${size}` : ''}`;

// Library
export const libraryList = () => request<{ libraries: LibraryItem[] }>('/library/list', { method: 'POST', body: '{}' });
export const libraryEnable = (id: number) => request('/library/enable', { method: 'POST', body: JSON.stringify({ id }) });
export const libraryDisable = (id: number) => request('/library/disable', { method: 'POST', body: JSON.stringify({ id }) });

// Search
export const search = (params: SearchParams) => request<SearchResults>('/search', { method: 'POST', body: JSON.stringify(params) });
export const searchSuggest = (query: string) => request<{ suggestions: Suggestion[] }>('/search/suggest', { method: 'POST', body: JSON.stringify({ query, limit: 8 }) });

// Bookmark
export const bookmarkList = () => request<{ bookmarks: BookmarkItem[] }>('/bookmark/list', { method: 'POST', body: '{}' });
export const bookmarkGet = (song_id: number, device_id?: string) => request<BookmarkItem>('/bookmark/get', { method: 'POST', body: JSON.stringify({ song_id, device_id }) });
export const bookmarkCreate = (data: { song_id: number; position_ms: number; device_id?: string }) => request<BookmarkItem>('/bookmark/create', { method: 'POST', body: JSON.stringify(data) });
export const bookmarkDelete = (id: number) => request('/bookmark/delete', { method: 'POST', body: JSON.stringify({ id }) });

// Annotation
export const toggleStar = (type: string, id: number) => request<{ starred: boolean }>('/annotation/star', { method: 'POST', body: JSON.stringify({ type, id }) });
export const setRating = (type: string, id: number, rating: number) => request<{ rating: number }>('/annotation/rate', { method: 'POST', body: JSON.stringify({ type, id, rating }) });
export const clearRating = (type: string, id: number) => request('/annotation/rate/clear', { method: 'POST', body: JSON.stringify({ type, id }) });
export const scrobble = (song_id: number, submission?: boolean) => request('/annotation/scrobble', { method: 'POST', body: JSON.stringify({ song_id, submission }) });
export const starredList = (offset?: number) => request<StarredResults>('/annotation/starred', { method: 'POST', body: JSON.stringify({ offset, limit: 50 }) });
export const ratedList = (offset?: number) => request<RatedResults>('/annotation/rated', { method: 'POST', body: JSON.stringify({ offset, limit: 50 }) });
export const playHistory = (offset?: number) => request<HistoryResults>('/annotation/history', { method: 'POST', body: JSON.stringify({ offset, limit: 50 }) });

// Share
export const shareList = () => request<{ shares: ShareItem[] }>('/share/list', { method: 'POST', body: '{}' });
export const shareCreate = (data: { type: string; item_id: number; description?: string; expires_in_days?: number }) => request<ShareItem>('/share/create', { method: 'POST', body: JSON.stringify(data) });
export const shareUpdate = (data: { id: number; description?: string | null; expires_in_days?: number }) => request('/share/update', { method: 'POST', body: JSON.stringify(data) });
export const shareDelete = (id: number) => request('/share/delete', { method: 'POST', body: JSON.stringify({ id }) });
export const shareDeleteBatch = (ids: number[]) => request<{ deleted: number }>('/share/delete/batch', { method: 'POST', body: JSON.stringify({ ids }) });

// Admin
export const adminUserList = () => request<{ users: AdminUser[] }>('/admin/user/list', { method: 'POST', body: '{}' });
export const adminUserAdd = (data: AdminAddUser) => request('/admin/user/add', { method: 'POST', body: JSON.stringify(data) });
export const adminUserDel = (id: number) => request('/admin/user/del', { method: 'POST', body: JSON.stringify({ id }) });
export const adminUserEdit = (data: { id: number; email?: string | null; scrobbling_enabled?: boolean; max_bit_rate?: number }) => request('/admin/user/edit', { method: 'POST', body: JSON.stringify(data) });
export const adminUserEnable = (id: number) => request('/admin/user/enable', { method: 'POST', body: JSON.stringify({ id }) });
export const adminUserDisable = (id: number) => request('/admin/user/disable', { method: 'POST', body: JSON.stringify({ id }) });
export const adminServerStatus = () => request<ServerStatus>('/admin/server/status');
export const adminLibraryAdd = (data: { name: string; path: string }) => request('/admin/library/add', { method: 'POST', body: JSON.stringify(data) });
export const adminLibraryDel = (id: number) => request('/admin/library/del', { method: 'POST', body: JSON.stringify({ id }) });
export const adminLibraryScan = (library_ids: number[]) => request('/admin/library/scan', { method: 'POST', body: JSON.stringify({ library_ids }) });

// ---- Types ----

export interface UserInfo {
  id: number; username: string; privs: Record<string, boolean>;
  email: string | null; scrobbling_enabled: boolean; max_bit_rate: number;
  created_at: number; updated_at?: number;
}

export interface EditUserRequest {
  email?: string | null; scrobbling_enabled?: boolean; max_bit_rate?: number;
}

export interface SessionInfo { id: string; created_at: number; last_used_at: number; device_info: string | null; }

export interface SongDetail { id: number; title: string; artist_id: number; artist_name: string; album_id: number | null; album_name: string | null; track_number: number | null; disc_number: number | null; duration_secs: number; bit_rate: number | null; size_bytes: number | null; file_format: string | null; content_type: string | null; year: number | null; play_count: number; created_at: number; starred: number | null; }

export interface SongItem { id: number; title: string; artist_name: string; album_name?: string | null; track_number?: number | null; duration_secs: number; starred: number | null; }

export interface LyricsData { type: string; lines: { start_ms: number | null; text: string }[]; }

export interface NowPlayingEntry { song_id: number; title: string; artist_name: string; device_id: string | null; minutes_ago: number; }

export interface MusicListParams { type: string; artist_id?: number; year_from?: number; year_to?: number; offset?: number; limit?: number; }

export interface AlbumDetail { id: number; name: string; artist_id: number; artist_name: string; year: number | null; song_count: number; duration_secs: number; play_count: number; cover_url: string; starred: number | null; songs: SongItem[]; }

export interface AlbumItem { id: number; name: string; artist_id: number; artist_name: string; year: number | null; song_count: number; duration_secs: number; cover_url: string; starred: number | null; }

export interface AlbumListParams { type: string; artist_id?: number; year_from?: number; year_to?: number; offset?: number; limit?: number; }

export interface ArtistDetail { id: number; name: string; sort_name?: string; album_count: number; song_count: number; play_count: number; starred: number | null; albums: AlbumItem[]; }

export interface ArtistItem { id: number; name: string; sort_name?: string; album_count: number; song_count: number; cover_url: string; starred: number | null; }

export interface ArtistListParams { letter?: string; offset?: number; limit?: number; }

export interface PlaylistSummary { id: number; name: string; owner_name: string; is_public: boolean; song_count: number; duration_secs: number; created_at: number; updated_at: number; }

export interface PlaylistDetail { id: number; name: string; owner_name: string; comment: string | null; is_public: boolean; song_count: number; duration_secs: number; created_at: number; updated_at: number; songs: PlaylistSongItem[]; }

export interface PlaylistSongItem { position: number; song_id: number; title: string; artist_name: string; album_name: string | null; duration_secs: number; starred: number | null; }

export interface LibraryItem { id: number; name: string; path: string; is_enabled: boolean; song_count: number; created_at: number; }

export interface SearchParams { query: string; type?: string; year_from?: number; year_to?: number; offset?: number; limit?: number; }

export interface SearchResults { artists: { id: number; name: string; album_count: number }[]; albums: { id: number; name: string; artist_name: string; year: number | null; song_count: number }[]; songs: { id: number; title: string; artist_name: string; album_name: string | null; track_number: number | null; duration_secs: number }[]; artist_total: number; album_total: number; song_total: number; }

export interface Suggestion { type: string; id: number; text: string; }

export interface BookmarkItem { id: number; song_id: number; title: string; artist_name: string; position_ms: number; device_id: string | null; updated_at: number; }

export interface StarredResults { artists: { id: number; name: string }[]; albums: { id: number; name: string; artist_name: string }[]; songs: { id: number; title: string; artist_name: string; album_name: string | null }[]; artist_total: number; album_total: number; song_total: number; }

export interface RatedResults { songs: { id: number; title: string; artist_name: string; rating: number }[]; albums: { id: number; name: string; artist_name: string; rating: number }[]; song_total: number; album_total: number; }

export interface HistoryResults { entries: { song_id: number; title: string; artist_name: string; album_name: string | null; played_at: number }[]; total: number; }

export interface ShareItem { id: number; type: string; item_id: number; title: string; description: string | null; token: string; url: string; visit_count: number; last_visited_at: number | null; expires_at: number | null; created_at: number; }

export interface AdminUser { id: number; username: string; privs: Record<string, boolean>; email: string | null; scrobbling_enabled: boolean; max_bit_rate: number; created_at: number; }

export interface AdminAddUser { username: string; password: string; email?: string; privs?: Record<string, boolean>; scrobbling_enabled?: boolean; max_bit_rate?: number; }

export interface ServerStatus { version: string; system: { memory_usage: number; memory_total: number | null; cpu_usage: number | null; uptime_secs: number; disk_total: number | null; disk_used: number | null }; database: { backend: string; connected: boolean }; cache: { backend: string }; server: { host: string; port: number }; }
