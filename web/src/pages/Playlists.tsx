import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Card, Button, Typography, Modal, Form, Input, Switch, List, Tag, Space, Spin, Empty, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { playlistList, playlistCreate, playlistDelete, playlistCoverUrl } from '../api/client';
import type { PlaylistSummary } from '../api/client';

const { Title, Text } = Typography;

export default function Playlists() {
  const [playlists, setPlaylists] = useState<PlaylistSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [form] = Form.useForm();
  const navigate = useNavigate();

  const load = () => playlistList().then(r => setPlaylists(r?.playlists ?? [])).catch(() => {}).finally(() => setLoading(false));

  useEffect(() => { load(); }, []);

  const handleCreate = async (values: { name: string; comment?: string; is_public?: boolean }) => {
    try {
      await playlistCreate(values);
      message.success('Playlist created');
      setOpen(false);
      form.resetFields();
      load();
    } catch { message.error('Failed to create'); }
  };

  const handleDelete = async (id: number) => {
    Modal.confirm({ title: 'Delete playlist?', onOk: async () => { await playlistDelete(id); load(); message.success('Deleted'); } });
  };

  const formatDuration = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toString().padStart(2, '0')}`;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <Title level={3} style={{ margin: 0 }}>Playlists</Title>
        <Button type="primary" icon={<PlusOutlined />} onClick={() => setOpen(true)}>New</Button>
      </div>
      <Modal title="New Playlist" open={open} onCancel={() => setOpen(false)} onOk={() => form.submit()} destroyOnClose>
        <Form form={form} layout="vertical" onFinish={handleCreate}>
          <Form.Item name="name" rules={[{ required: true, message: 'Name required' }]}>
            <Input placeholder="Playlist name" />
          </Form.Item>
          <Form.Item name="comment"><Input placeholder="Comment (optional)" /></Form.Item>
          <Form.Item name="is_public" valuePropName="checked"><Switch /> Public</Form.Item>
        </Form>
      </Modal>
      {loading ? <Spin size="large" style={{ display: 'block', margin: '40px auto' }} /> : (
        playlists.length === 0 ? <Empty description="No playlists" /> : (
          <List dataSource={playlists} renderItem={p => (
            <Card hoverable size="small" style={{ marginBottom: 8 }} onClick={() => navigate(`/playlists/${p.id}`)}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <Space>
                  <img src={playlistCoverUrl(p.id, 48)} alt="" style={{ width: 48, height: 48, borderRadius: 6, objectFit: 'cover' }} />
                  <div>
                    <Text strong>{p.name}</Text>
                    <br />
                    <Text type="secondary" style={{ fontSize: 12 }}>{p.owner_name} · {p.song_count} songs · {formatDuration(p.duration_secs)}</Text>
                  </div>
                </Space>
                <Space>
                  {p.is_public && <Tag color="green">Public</Tag>}
                  <Button size="small" danger onClick={e => { e.stopPropagation(); handleDelete(p.id); }}>Del</Button>
                </Space>
              </div>
            </Card>
          )} />
        )
      )}
    </div>
  );
}
