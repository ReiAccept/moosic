import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, Row, Col, Spin, Empty, Typography, Segmented } from 'antd';
import { albumList, albumCoverUrl } from '../api/client';
import type { AlbumItem } from '../api/client';

const { Title } = Typography;
const TYPES = ['newest', 'recent', 'frequent', 'random', 'alphabeticalByName', 'starred'];

export default function Albums() {
  const [albums, setAlbums] = useState<AlbumItem[]>([]);
  const [sortType, setSortType] = useState('newest');
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    setLoading(true);
    albumList({ type: sortType, limit: 48 })
      .then(r => setAlbums(r?.albums ?? []))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [sortType]);

  return (
    <div>
      <Title level={3}>Albums</Title>
      <Segmented options={TYPES} value={sortType} onChange={v => setSortType(v as string)} style={{ marginBottom: 16 }} />
      {loading ? <Spin size="large" style={{ display: 'block', margin: '40px auto' }} /> : (
        albums.length === 0 ? <Empty description="No albums" /> : (
          <Row gutter={[16, 16]}>
            {albums.map(a => (
              <Col key={a.id} xs={12} sm={8} md={6} lg={4} xl={3}>
                <Card hoverable cover={<img className="cover-img" src={albumCoverUrl(a.id, 200)} alt={a.name} />}
                  onClick={() => navigate(`/albums/${a.id}`)} size="small">
                  <Card.Meta title={a.name} description={`${a.artist_name} · ${a.year || '—'}`} />
                </Card>
              </Col>
            ))}
          </Row>
        )
      )}
    </div>
  );
}
