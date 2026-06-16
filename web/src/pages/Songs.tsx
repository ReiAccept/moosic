import { useState, useEffect, useRef } from 'react';
import { Table, Button, Typography, Segmented, Spin, Tag } from 'antd';
import { PlayCircleOutlined, PauseCircleOutlined } from '@ant-design/icons';
import { musicList, musicStreamUrl, scrobble } from '../api/client';
import type { SongItem } from '../api/client';

const { Title, Text } = Typography;
const TYPES = ['newest', 'recent', 'frequent', 'random', 'starred'];

export default function Songs() {
  const [songs, setSongs] = useState<SongItem[]>([]);
  const [sortType, setSortType] = useState('newest');
  const [loading, setLoading] = useState(false);
  const [playingId, setPlayingId] = useState<number | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    setLoading(true);
    musicList({ type: sortType, limit: 100 })
      .then(r => setSongs(r?.songs ?? []))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [sortType]);

  const togglePlay = (song: SongItem) => {
    if (playingId === song.id) {
      audioRef.current?.pause();
      setPlayingId(null);
      return;
    }
    if (audioRef.current) audioRef.current.pause();
    const audio = new Audio(musicStreamUrl(song.id));
    audio.play().catch(() => {});
    audioRef.current = audio;
    setPlayingId(song.id);
    scrobble(song.id, false).catch(() => {});
    audio.onended = () => { scrobble(song.id, true).catch(() => {}); setPlayingId(null); };
  };

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  const columns = [
    { title: '', width: 48, render: (_: any, r: SongItem) => (
      <Button type="text" size="small" icon={playingId === r.id ? <PauseCircleOutlined /> : <PlayCircleOutlined />}
        onClick={() => togglePlay(r)} />
    )},
    { title: 'Title', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Album', dataIndex: 'album_name', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Duration', dataIndex: 'duration_secs', render: (v: number) => <Text type="secondary">{formatDuration(v)}</Text> },
  ];

  return (
    <div>
      <Title level={3}>Songs</Title>
      {playingId && (
        <Tag color="#e94560" style={{ marginBottom: 12, padding: '4px 12px' }}>
          Now Playing: {songs.find(s => s.id === playingId)?.title}
        </Tag>
      )}
      <Segmented options={TYPES} value={sortType} onChange={v => setSortType(v as string)} style={{ marginBottom: 16 }} />
      {loading ? <Spin size="large" style={{ display: 'block', margin: '40px auto' }} /> : (
        <Table columns={columns} dataSource={songs} rowKey="id" pagination={{ pageSize: 50 }} size="small" />
      )}
    </div>
  );
}
