import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, Button, Row, Col, Spin, Empty, Typography, Space } from 'antd';
import { artistList, artistCoverUrl } from '../api/client';
import type { ArtistItem } from '../api/client';

const { Title } = Typography;
const LETTERS = '#ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');

export default function Artists() {
  const [artists, setArtists] = useState<ArtistItem[]>([]);
  const [letter, setLetter] = useState('');
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    setLoading(true);
    artistList({ letter: letter || undefined, limit: 100 })
      .then(r => setArtists(r?.artists ?? []))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [letter]);

  return (
    <div>
      <Title level={3}>Artists</Title>
      <Space wrap style={{ marginBottom: 16 }}>
        {LETTERS.map(l => (
          <Button key={l} type={letter === l ? 'primary' : 'default'} size="small" onClick={() => setLetter(letter === l ? '' : l)}>
            {l}
          </Button>
        ))}
      </Space>
      {loading ? <Spin size="large" style={{ display: 'block', margin: '40px auto' }} /> : (
        artists.length === 0 ? <Empty description="No artists found" /> : (
          <Row gutter={[16, 16]}>
            {artists.map(a => (
              <Col key={a.id} xs={12} sm={8} md={6} lg={4} xl={3}>
                <Card hoverable cover={<img className="cover-img" src={artistCoverUrl(a.id, 200)} alt={a.name} />}
                  onClick={() => navigate(`/artists/${a.id}`)} size="small">
                  <Card.Meta title={a.name} description={`${a.album_count} albums · ${a.song_count} songs`} />
                </Card>
              </Col>
            ))}
          </Row>
        )
      )}
    </div>
  );
}
