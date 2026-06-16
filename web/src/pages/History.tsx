import { useState, useEffect } from 'react';
import { Table, Typography, Empty, Spin } from 'antd';
import { playHistory } from '../api/client';

const { Title, Text } = Typography;

export default function History() {
  const [entries, setEntries] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    playHistory().then(r => setEntries(r?.entries ?? [])).catch(() => {}).finally(() => setLoading(false));
  }, []);

  const columns = [
    { title: 'Song', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Artist', dataIndex: 'artist_name' },
    { title: 'Album', dataIndex: 'album_name', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Played', dataIndex: 'played_at', render: (v: number) => <Text type="secondary">{new Date(v).toLocaleString()}</Text> },
  ];

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <Title level={3}>History</Title>
      {entries.length === 0 ? <Empty description="No history" /> : (
        <Table columns={columns} dataSource={entries} rowKey={(_, i) => `${i}`} pagination={{ pageSize: 30 }} size="small" />
      )}
    </div>
  );
}
