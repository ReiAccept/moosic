import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, Table, Typography, Row, Col, Spin, Empty, Tag } from 'antd';
import { PlayCircleOutlined } from '@ant-design/icons';
import { musicRand, musicPlaying, albumList, albumCoverUrl } from '../api/client';
import type { SongItem, AlbumItem, NowPlayingEntry } from '../api/client';

const { Title, Text } = Typography;

export default function Home() {
  const [randomSongs, setRandomSongs] = useState<SongItem[]>([]);
  const [newAlbums, setNewAlbums] = useState<AlbumItem[]>([]);
  const [playing, setPlaying] = useState<NowPlayingEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    Promise.all([
      musicRand(12).then(r => setRandomSongs(r?.songs ?? [])),
      albumList({ type: 'newest', limit: 8 }).then(r => setNewAlbums(r?.albums ?? [])),
      musicPlaying().then(r => setPlaying(r?.entries ?? [])),
    ]).catch(() => {}).finally(() => setLoading(false));
  }, []);

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  const songColumns = [
    { title: '#', dataIndex: 'idx', width: 50, render: (_: any, __: any, i: number) => <Text type="secondary">{i + 1}</Text> },
    { title: 'Title', dataIndex: 'title', render: (t: string, r: SongItem) => <a onClick={() => navigate(`/albums/${(r as any).album_id || 0}`)}>{t}</a> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Album', dataIndex: 'album_name', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Duration', dataIndex: 'duration_secs', render: (v: number) => <Text type="secondary">{formatDuration(v)}</Text> },
  ];

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <Title level={3}>Home</Title>

      {playing.length > 0 && (
        <Card title="Now Playing" size="small" style={{ marginBottom: 24 }}>
          {playing.map(p => (
            <div key={p.song_id} style={{ padding: '4px 0', display: 'flex', alignItems: 'center', gap: 8 }}>
              <PlayCircleOutlined />
              <Text>{p.title} — {p.artist_name}</Text>
              <Tag>{p.minutes_ago}m ago</Tag>
            </div>
          ))}
        </Card>
      )}

      <Title level={5} style={{ marginBottom: 12 }}>New Albums</Title>
      {newAlbums.length === 0 ? <Empty description="No albums" /> : (
        <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
          {newAlbums.map(a => (
            <Col key={a.id} xs={12} sm={8} md={6} lg={4} xl={3}>
              <Card
                hoverable
                cover={<img className="cover-img" src={albumCoverUrl(a.id, 200)} alt={a.name} />}
                onClick={() => navigate(`/albums/${a.id}`)}
                size="small"
              >
                <Card.Meta title={a.name} description={a.artist_name} />
              </Card>
            </Col>
          ))}
        </Row>
      )}

      <Title level={5} style={{ marginBottom: 12 }}>Random Songs</Title>
      <Table columns={songColumns} dataSource={randomSongs} rowKey="id" pagination={false} size="small" />
    </div>
  );
}
