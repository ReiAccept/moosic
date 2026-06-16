import { useState, useEffect, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { Table, Button, Typography, Spin, Tag, Flex, message } from 'antd';
import { PlayCircleOutlined, PauseCircleOutlined, DeleteOutlined } from '@ant-design/icons';
import { playlistInfo, playlistRemoveSongs, playlistCoverUrl, musicStreamUrl, scrobble } from '../api/client';
import type { PlaylistDetail, PlaylistSongItem } from '../api/client';

const { Title, Text } = Typography;

export default function PlaylistDetail() {
  const { id } = useParams<{ id: string }>();
  const [playlist, setPlaylist] = useState<PlaylistDetail | null>(null);
  const [playingId, setPlayingId] = useState<number | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const load = () => {
    if (!id) return;
    playlistInfo(+id).then(setPlaylist).catch(() => {});
  };

  useEffect(() => { load(); }, [id]);

  const togglePlay = (song: PlaylistSongItem) => {
    if (playingId === song.song_id) { audioRef.current?.pause(); setPlayingId(null); return; }
    if (audioRef.current) audioRef.current.pause();
    const a = new Audio(musicStreamUrl(song.song_id));
    a.play().catch(() => {});
    audioRef.current = a;
    setPlayingId(song.song_id);
    scrobble(song.song_id, false).catch(() => {});
    a.onended = () => { scrobble(song.song_id, true).catch(() => {}); setPlayingId(null); };
  };

  const handleRemove = async (songId: number) => {
    if (!playlist) return;
    try { await playlistRemoveSongs(playlist.id, [songId]); load(); message.success('Removed'); } catch { message.error('Failed'); }
  };

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  const columns = [
    { title: '#', dataIndex: 'position', width: 50, render: (v: number) => <Text type="secondary">{v}</Text> },
    { title: '', width: 48, render: (_: any, r: PlaylistSongItem) => (
      <Button type="text" size="small" icon={playingId === r.song_id ? <PauseCircleOutlined /> : <PlayCircleOutlined />}
        onClick={() => togglePlay(r)} />
    )},
    { title: 'Title', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Album', dataIndex: 'album_name', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Duration', dataIndex: 'duration_secs', render: (v: number) => <Text type="secondary">{formatDuration(v)}</Text> },
    { title: '', width: 48, render: (_: any, r: PlaylistSongItem) => (
      <Button type="text" size="small" danger icon={<DeleteOutlined />} onClick={() => handleRemove(r.song_id)} />
    )},
  ];

  if (!playlist) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <Flex gap={24} style={{ marginBottom: 24 }}>
        <img src={playlistCoverUrl(playlist.id, 180)} alt="" style={{ width: 180, height: 180, borderRadius: 10, objectFit: 'cover' }} />
        <div>
          <Title level={2} style={{ marginBottom: 4 }}>{playlist.name}</Title>
          <Text type="secondary">{playlist.owner_name} · {playlist.is_public ? <Tag color="green">Public</Tag> : <Tag>Private</Tag>}</Text>
          {playlist.comment && <><br /><Text>{playlist.comment}</Text></>}
          <br />
          <Text type="secondary">{playlist.song_count} songs · {formatDuration(playlist.duration_secs)}</Text>
        </div>
      </Flex>
      <Table columns={columns} dataSource={playlist.songs} rowKey={r => `${r.song_id}-${r.position}`} pagination={false} size="small" />
    </div>
  );
}
