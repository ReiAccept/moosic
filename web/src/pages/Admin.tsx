import { useState, useEffect } from 'react';
import { Table, Button, Typography, Tabs, Modal, Form, Input, Card, Tag, Space, Spin, message, Descriptions } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import { adminUserList, adminUserAdd, adminUserDel, adminUserEnable, adminUserDisable, adminServerStatus, adminLibraryAdd, adminLibraryDel, adminLibraryScan, libraryList } from '../api/client';
import type { AdminUser, ServerStatus, LibraryItem } from '../api/client';

const { Title, Text } = Typography;

export default function Admin() {
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [libraries, setLibraries] = useState<LibraryItem[]>([]);
  const [server, setServer] = useState<ServerStatus | null>(null);
  const [userOpen, setUserOpen] = useState(false);
  const [libOpen, setLibOpen] = useState(false);
  const [userForm] = Form.useForm();
  const [libForm] = Form.useForm();

  const loadUsers = () => adminUserList().then(r => setUsers(r?.users ?? [])).catch(() => {});
  const loadLibs = () => libraryList().then(r => setLibraries(r?.libraries ?? [])).catch(() => {});
  const loadServer = () => adminServerStatus().then(setServer).catch(() => {});

  useEffect(() => { loadUsers(); loadLibs(); loadServer(); }, []);

  const handleAddUser = async (values: any) => {
    try { await adminUserAdd(values); message.success('User created'); setUserOpen(false); userForm.resetFields(); loadUsers(); } catch { message.error('Failed'); }
  };

  const handleAddLib = async (values: any) => {
    try { await adminLibraryAdd(values); message.success('Library added'); setLibOpen(false); libForm.resetFields(); loadLibs(); } catch { message.error('Failed'); }
  };

  const formatBytes = (b: number) => b < 1024 ? `${b} B` : b < 1048576 ? `${(b / 1024).toFixed(1)} KB` : `${(b / 1048576).toFixed(1)} MB`;

  const userColumns = [
    { title: 'ID', dataIndex: 'id', width: 60 },
    { title: 'Username', dataIndex: 'username', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Email', dataIndex: 'email', render: (v: string | null) => <Text type="secondary">{v || '—'}</Text> },
    { title: 'Scrobbling', dataIndex: 'scrobbling_enabled', render: (v: boolean) => <Tag color={v ? 'green' : 'default'}>{v ? 'On' : 'Off'}</Tag> },
    { title: 'Role', render: (_: any, u: AdminUser) => {
      const p = u.privs;
      const isAdmin = p != null && typeof p === 'object' && !Array.isArray(p) && Object.keys(p).length > 0;
      return <Tag color={isAdmin ? 'green' : 'default'}>{isAdmin ? 'Admin' : 'User'}</Tag>;
    }},
    { title: '', render: (_: any, u: AdminUser) => (
      <Space>
        <Button size="small" onClick={() => { adminUserEnable(u.id); loadUsers(); }}>Enable</Button>
        <Button size="small" onClick={() => { adminUserDisable(u.id); loadUsers(); }}>Disable</Button>
        <Button size="small" danger onClick={() => {
          Modal.confirm({ title: `Delete ${u.username}?`, onOk: () => { adminUserDel(u.id); loadUsers(); message.success('Deleted'); } });
        }}>Del</Button>
      </Space>
    )},
  ];

  const libColumns = [
    { title: 'ID', dataIndex: 'id', width: 60 },
    { title: 'Name', dataIndex: 'name', render: (t: string) => <Text strong>{t}</Text> },
    { title: 'Path', dataIndex: 'path', render: (v: string) => <Text code style={{ fontSize: 11 }}>{v}</Text> },
    { title: 'Songs', dataIndex: 'song_count' },
    { title: '', render: (_: any, l: LibraryItem) => (
      <Space>
        <Button type="primary" size="small" onClick={() => { adminLibraryScan([l.id]); message.info('Scan started'); }}>Scan</Button>
        <Button size="small" danger onClick={() => {
          Modal.confirm({ title: `Delete ${l.name}?`, onOk: () => { adminLibraryDel(l.id); loadLibs(); message.success('Deleted'); } });
        }}>Del</Button>
      </Space>
    )},
  ];

  return (
    <div>
      <Title level={3}>Admin</Title>
      <Tabs items={[
        {
          key: 'users', label: 'Users',
          children: (
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 12 }}>
                <Text>{users.length} users</Text>
                <Button type="primary" icon={<PlusOutlined />} size="small" onClick={() => setUserOpen(true)}>Add User</Button>
              </div>
              <Modal title="Add User" open={userOpen} onCancel={() => setUserOpen(false)} onOk={() => userForm.submit()} destroyOnClose>
                <Form form={userForm} layout="vertical" onFinish={handleAddUser}>
                  <Form.Item name="username" rules={[{ required: true }]}><Input placeholder="Username" /></Form.Item>
                  <Form.Item name="password" rules={[{ required: true }]}><Input.Password placeholder="Password" /></Form.Item>
                  <Form.Item name="email"><Input placeholder="Email" /></Form.Item>
                </Form>
              </Modal>
              <Table columns={userColumns} dataSource={users} rowKey="id" pagination={false} size="small" />
            </div>
          ),
        },
        {
          key: 'libraries', label: 'Libraries',
          children: (
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 12 }}>
                <Text>{libraries.length} libraries</Text>
                <Button type="primary" icon={<PlusOutlined />} size="small" onClick={() => setLibOpen(true)}>Add Library</Button>
              </div>
              <Modal title="Add Library" open={libOpen} onCancel={() => setLibOpen(false)} onOk={() => libForm.submit()} destroyOnClose>
                <Form form={libForm} layout="vertical" onFinish={handleAddLib}>
                  <Form.Item name="name" rules={[{ required: true }]}><Input placeholder="Name" /></Form.Item>
                  <Form.Item name="path" rules={[{ required: true }]}><Input placeholder="Path (e.g. /data/music)" /></Form.Item>
                </Form>
              </Modal>
              <Table columns={libColumns} dataSource={libraries} rowKey="id" pagination={false} size="small" />
            </div>
          ),
        },
        {
          key: 'server', label: 'Server',
          children: server ? (
            <Card>
              <Descriptions column={1} bordered size="small">
                <Descriptions.Item label="Version">{server.version}</Descriptions.Item>
                <Descriptions.Item label="Database">{server.database.backend} — {server.database.connected ? <Tag color="green">Connected</Tag> : <Tag color="red">Disconnected</Tag>}</Descriptions.Item>
                <Descriptions.Item label="Cache">{server.cache.backend}</Descriptions.Item>
                <Descriptions.Item label="Listen">{server.server.host}:{server.server.port}</Descriptions.Item>
                <Descriptions.Item label="Uptime">{Math.floor(server.system.uptime_secs / 3600)}h {Math.floor((server.system.uptime_secs % 3600) / 60)}m</Descriptions.Item>
                <Descriptions.Item label="Memory">{formatBytes(server.system.memory_usage)} / {server.system.memory_total ? formatBytes(server.system.memory_total) : 'N/A'}</Descriptions.Item>
              </Descriptions>
            </Card>
          ) : <Spin />,
        },
      ]} />
    </div>
  );
}
