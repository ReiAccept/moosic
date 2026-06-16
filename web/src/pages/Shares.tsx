import { useState, useEffect } from 'react';
import { Table, Button, Typography, Modal, Form, Input, InputNumber, Select, Space, Tag, Spin, message } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { shareList, shareCreate, shareDelete, shareDeleteBatch } from '../api/client';
import type { ShareItem } from '../api/client';

const { Title, Text } = Typography;

export default function Shares() {
  const [shares, setShares] = useState<ShareItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [open, setOpen] = useState(false);
  const [selected, setSelected] = useState<number[]>([]);
  const [form] = Form.useForm();

  const load = () => {
    setLoading(true);
    shareList().then(r => setShares(r?.shares ?? [])).catch(() => {}).finally(() => setLoading(false));
  };

  useEffect(() => { load(); }, []);

  const handleCreate = async (values: any) => {
    try {
      await shareCreate({ type: values.type, item_id: values.item_id, description: values.description, expires_in_days: values.expires_in_days });
      message.success('Share created');
      setOpen(false);
      form.resetFields();
      load();
    } catch { message.error('Failed'); }
  };

  const handleBatchDelete = async () => {
    if (selected.length === 0) return;
    Modal.confirm({
      title: `Delete ${selected.length} share(s)?`,
      onOk: async () => { await shareDeleteBatch(selected); setSelected([]); load(); message.success('Deleted'); },
    });
  };

  const columns = [
    { title: 'Title', dataIndex: 'title', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Type', dataIndex: 'type', render: (v: string) => <Tag>{v}</Tag> },
    { title: 'Token', dataIndex: 'token', render: (v: string) => <Text code style={{ fontSize: 11 }}>{v.substring(0, 12)}...</Text> },
    { title: 'Visits', dataIndex: 'visit_count' },
    { title: 'Expires', dataIndex: 'expires_at', render: (v: number | null) => v ? new Date(v).toLocaleDateString() : <Tag color="blue">Never</Tag> },
    { title: '', width: 48, render: (_: any, r: ShareItem) => (
      <Button size="small" danger onClick={() => { shareDelete(r.id); load(); }}>Del</Button>
    )},
  ];

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '80px auto' }} />;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
        <Title level={3} style={{ margin: 0 }}>Shares</Title>
        <Space>
          {selected.length > 0 && <Button danger onClick={handleBatchDelete}>Delete {selected.length}</Button>}
          <Button type="primary" icon={<PlusOutlined />} onClick={() => setOpen(true)}>New</Button>
        </Space>
      </div>
      <Modal title="New Share" open={open} onCancel={() => setOpen(false)} onOk={() => form.submit()} destroyOnClose>
        <Form form={form} layout="vertical" onFinish={handleCreate}>
          <Form.Item name="type" rules={[{ required: true }]} initialValue="song">
            <Select options={[{ value: 'song', label: 'Song' }, { value: 'album', label: 'Album' }, { value: 'playlist', label: 'Playlist' }]} />
          </Form.Item>
          <Form.Item name="item_id" rules={[{ required: true, message: 'Required' }]}>
            <Input placeholder="Item ID" type="number" />
          </Form.Item>
          <Form.Item name="description"><Input placeholder="Description" /></Form.Item>
          <Form.Item name="expires_in_days"><InputNumber placeholder="Expires in days" min={1} style={{ width: '100%' }} /></Form.Item>
        </Form>
      </Modal>
      <Table columns={columns} dataSource={shares} rowKey="id" pagination={false} size="small"
        rowSelection={{ selectedRowKeys: selected, onChange: keys => setSelected(keys as number[]) }} />
    </div>
  );
}
