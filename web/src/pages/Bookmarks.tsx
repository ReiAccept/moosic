import { useState, useEffect, useRef } from 'react';
import { Table, Button, Typography, Empty, Spin } from 'antd';
import { PlayCircleOutlined, DeleteOutlined } from '@ant-design/icons';
import { bookmarkList, bookmarkDelete, musicStreamUrl, scrobble } from '../api/client';
import type { BookmarkItem } from '../api/client';

const { Title, Text } = Typography;

export default function Bookmarks() {
  const [bookmarks, setBookmarks] = useState<BookmarkItem[]>([]);
  const [loading, setLoading] = useState(true);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const load = () => {
    setLoading(true);
    bookmarkList().then(r => setBookmarks(r?.bookmarks ?? [])).catch(() => {}).finally(() => setLoading(false));
  };

  useEffect(() => { load(); }, []);

  const play = (b: BookmarkItem) => {
    if (audioRef.current) audioRef.current.pause();
    const audio = new Audio(musicStreamUrl(b.song_id));
    audio.currentTime = b.position_ms / 1000;
    audio.play().catch(() => {});
    audioRef.current = audio;
    scrobble(b.song_id, false).catch(() => {});
  };

  const formatPosition = (ms: number) => `${Math.floor(ms / 60000)}:${Math.floor((ms % 60000) / 1000).toString().padStart(2, '0')}`;

  const columns = [
    { title: '', width: 48, render: (_: any, r: BookmarkItem) => <Button type="text" size="small" icon={<PlayCircleOutlined />} onClick={() => play(r)} /> },
    { title: 'Song', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Position', dataIndex: 'position_ms', render: (v: number) => <Text type="secondary">{formatPosition(v)}</Text> },
    { title: 'Device', dataIndex: 'device_id', render: (v: string | null) => <Text type="secondary">{v || 'default'}</Text> },
    { title: '', width: 48, render: (_: any, r: BookmarkItem) => (
      <Button type="text" size="small" danger icon={<DeleteOutlined />} onClick={() => { bookmarkDelete(r.id); load(); }} />
    )},
  ];

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <Title level={3}>Bookmarks</Title>
      {bookmarks.length === 0 ? <Empty description="No bookmarks" /> : (
        <Table columns={columns} dataSource={bookmarks} rowKey="id" pagination={false} size="small" />
      )}
    </div>
  );
}
