import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Input, Table, Button, Typography, Spin, Card, Tag, Space, Tabs, Empty } from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import { search, searchSuggest } from '../api/client';
import type { SearchResults, Suggestion } from '../api/client';

const { Title, Text } = Typography;

export default function Search() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResults | null>(null);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    if (query.length < 2) { setSuggestions([]); return; }
    const timer = setTimeout(() => {
      searchSuggest(query).then(r => setSuggestions(r?.suggestions ?? [])).catch(() => {});
    }, 300);
    return () => clearTimeout(timer);
  }, [query]);

  const doSearch = useCallback(() => {
    if (!query.trim()) return;
    setLoading(true);
    search({ query: query.trim() }).then(setResults).catch(() => {}).finally(() => setLoading(false));
  }, [query]);

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  const songColumns = [
    { title: 'Title', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Album', dataIndex: 'album_name', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Duration', dataIndex: 'duration_secs', render: (v: number) => <Text type="secondary">{formatDuration(v)}</Text> },
  ];

  const albumColumns = [
    { title: 'Name', dataIndex: 'name', render: (t: string, r: any) => <a onClick={() => navigate(`/albums/${r.id}`)}>{t}</a> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Year', dataIndex: 'year', render: (v: number | null) => <Text type="secondary">{v || '—'}</Text> },
  ];

  return (
    <div>
      <Title level={3}>Search</Title>
      <Space.Compact style={{ width: '100%', maxWidth: 500, marginBottom: 16 }}>
        <Input placeholder="Search artists, albums, songs..." value={query} onChange={e => setQuery(e.target.value)}
          onPressEnter={doSearch} prefix={<SearchOutlined />} allowClear />
        <Button type="primary" onClick={doSearch} loading={loading}>Search</Button>
      </Space.Compact>
      {suggestions.length > 0 && !results && (
        <Card size="small" style={{ maxWidth: 500, marginBottom: 16 }}>
          {suggestions.map((s, i) => (
            <div key={i} style={{ padding: '4px 0', cursor: 'pointer' }}
              onClick={() => {
                if (s.type === 'artist') navigate(`/artists/${s.id}`);
                else if (s.type === 'album') navigate(`/albums/${s.id}`);
              }}>
              <Tag style={{ marginRight: 8 }}>{s.type}</Tag>{s.text}
            </div>
          ))}
        </Card>
      )}
      {loading && <Spin size="large" style={{ display: 'block', margin: '40px auto' }} />}
      {results && (
        results.artist_total === 0 && results.album_total === 0 && results.song_total === 0
          ? <Empty description="No results" />
          : <Tabs items={[
            { key: 'artists', label: `Artists (${results.artist_total})`, children: (
              <Space wrap>{results.artists.map(a => (
                <Card key={a.id} size="small" hoverable onClick={() => navigate(`/artists/${a.id}`)}>
                  <Text strong>{a.name}</Text><br /><Text type="secondary">{a.album_count} albums</Text>
                </Card>
              ))}</Space>
            )},
            { key: 'albums', label: `Albums (${results.album_total})`, children: (
              <Table columns={albumColumns} dataSource={results.albums} rowKey="id" pagination={false} size="small" />
            )},
            { key: 'songs', label: `Songs (${results.song_total})`, children: (
              <Table columns={songColumns} dataSource={results.songs} rowKey="id" pagination={false} size="small" />
            )},
          ].filter(t => t.children)} />
      )}
    </div>
  );
}
