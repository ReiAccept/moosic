import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Table, Button, Typography, Spin, Rate, Space, Flex, message } from 'antd';
import { StarOutlined, StarFilled } from '@ant-design/icons';
import { albumInfo, albumCoverUrl, toggleStar, setRating } from '../api/client';
import type { AlbumDetail } from '../api/client';

const { Title, Text } = Typography;

export default function AlbumDetail() {
  const { id } = useParams<{ id: string }>();
  const [album, setAlbum] = useState<AlbumDetail | null>(null);
  const [starred, setStarred] = useState(false);
  const [rating, setRatingState] = useState<number>(0);
  const navigate = useNavigate();

  useEffect(() => {
    if (!id) return;
    albumInfo(+id).then(a => { setAlbum(a); setStarred(!!a.starred); }).catch(() => {});
  }, [id]);

  const handleStar = async () => {
    if (!album) return;
    try { const r = await toggleStar('album', album.id); setStarred(r.starred); message.success(r.starred ? 'Starred' : 'Unstarred'); } catch { message.error('Failed'); }
  };

  const handleRate = async (r: number) => {
    if (!album) return;
    try { await setRating('album', album.id, r); setRatingState(r); message.success('Rated'); } catch { message.error('Failed'); }
  };

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  const songColumns = [
    { title: '#', width: 50, render: (_: any, r: any, i: number) => <Text type="secondary">{r.track_number || i + 1}</Text> },
    { title: 'Title', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Artist', dataIndex: 'artist_name', render: (v: string) => <Text type="secondary">{v}</Text> },
    { title: 'Duration', dataIndex: 'duration_secs', render: (v: number) => <Text type="secondary">{formatDuration(v)}</Text> },
  ];

  if (!album) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <Flex gap={24} style={{ marginBottom: 24 }}>
        <img src={albumCoverUrl(album.id, 250)} alt={album.name} style={{ width: 200, height: 200, borderRadius: 10, objectFit: 'cover' }} />
        <div>
          <Title level={2} style={{ marginBottom: 4 }}>{album.name}</Title>
          <Text style={{ fontSize: 16 }}>
            <a onClick={() => navigate(`/artists/${album.artist_id}`)}>{album.artist_name}</a>
          </Text>
          <br />
          <Text type="secondary">{album.year || '—'} · {album.song_count} songs · {formatDuration(album.duration_secs)} · {album.play_count} plays</Text>
          <br />
          <Space style={{ marginTop: 12 }}>
            <Button icon={starred ? <StarFilled /> : <StarOutlined />} type={starred ? 'primary' : 'default'} onClick={handleStar}>
              {starred ? 'Starred' : 'Star'}
            </Button>
            <Text>Rate:</Text>
            <Rate value={rating} onChange={handleRate} />
          </Space>
        </div>
      </Flex>
      <Title level={4} style={{ marginBottom: 12 }}>Songs</Title>
      <Table columns={songColumns} dataSource={album.songs} rowKey="id" pagination={false} size="small" />
    </div>
  );
}
