import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Card, Table, Button, Row, Col, Spin, Typography, App } from 'antd';
import { StarOutlined, StarFilled } from '@ant-design/icons';
import { artistInfo, artistSongs, artistCoverUrl, albumCoverUrl, toggleStar } from '../api/client';
import type { ArtistDetail, SongItem } from '../api/client';

const { Title, Text } = Typography;

export default function ArtistDetail() {
  const { id } = useParams<{ id: string }>();
  const [artist, setArtist] = useState<ArtistDetail | null>(null);
  const [songs, setSongs] = useState<SongItem[]>([]);
  const [starred, setStarred] = useState(false);
  const navigate = useNavigate();
  const { message } = App.useApp();

  useEffect(() => {
    if (!id) return;
    artistInfo(+id).then(a => { setArtist(a); setStarred(!!a.starred); }).catch(() => {});
    artistSongs(+id).then(r => setSongs(r?.songs ?? [])).catch(() => {});
  }, [id]);

  const handleStar = async () => {
    if (!artist) return;
    try {
      const r = await toggleStar('artist', artist.id);
      setStarred(r.starred);
      message.success(r.starred ? 'Starred' : 'Unstarred');
    } catch { message.error('Failed'); }
  };

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  const songColumns = [
    { title: '#', width: 50, render: (_: any, __: any, i: number) => <Text type="secondary">{i + 1}</Text> },
    { title: 'Title', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Album', dataIndex: 'album_name', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Duration', dataIndex: 'duration_secs', render: (v: number) => <Text type="secondary">{formatDuration(v)}</Text> },
  ];

  if (!artist) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <div style={{ display: 'flex', gap: 24, marginBottom: 24 }}>
        <img src={artistCoverUrl(artist.id, 200)} alt={artist.name} style={{ width: 180, height: 180, borderRadius: 10, objectFit: 'cover' }} />
        <div>
          <Title level={2} style={{ marginBottom: 4 }}>{artist.name}</Title>
          <Text type="secondary">{artist.album_count} albums · {artist.song_count} songs · {artist.play_count} plays</Text>
          <br />
          <Button icon={starred ? <StarFilled /> : <StarOutlined />} type={starred ? 'primary' : 'default'}
            onClick={handleStar} style={{ marginTop: 12 }}>
            {starred ? 'Starred' : 'Star'}
          </Button>
        </div>
      </div>

      <Title level={4} style={{ marginBottom: 12 }}>Albums</Title>
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        {artist.albums.map(a => (
          <Col key={a.id} xs={12} sm={8} md={6} lg={4} xl={3}>
            <Card hoverable cover={<img className="cover-img" src={albumCoverUrl(a.id, 150)} alt={a.name} />}
              onClick={() => navigate(`/albums/${a.id}`)} size="small">
              <Card.Meta title={a.name} description={`${a.year || '—'} · ${a.song_count} songs`} />
            </Card>
          </Col>
        ))}
      </Row>

      <Title level={4} style={{ marginBottom: 12 }}>Songs</Title>
      <Table columns={songColumns} dataSource={songs} rowKey="id" pagination={false} size="small" />
    </div>
  );
}
