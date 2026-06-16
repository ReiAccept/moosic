import { useNavigate, useLocation } from 'react-router-dom';
import { Layout, Menu, Button, Typography } from 'antd';
import {
  HomeOutlined,
  CustomerServiceOutlined,
  PlayCircleOutlined,
  UnorderedListOutlined,
  SearchOutlined,
  BookOutlined,
  ShareAltOutlined,
  ClockCircleOutlined,
  SettingOutlined,
  LogoutOutlined,
} from '@ant-design/icons';

const { Sider } = Layout;
const { Text } = Typography;

function getSavedUser() {
  const raw = localStorage.getItem('moosic_user');
  return raw ? JSON.parse(raw) : null;
}

export default function Sidebar() {
  const location = useLocation();
  const navigate = useNavigate();
  const user = getSavedUser();
  const privs = user?.privs;
  const isAdmin = privs != null && typeof privs === 'object' && !Array.isArray(privs) && Object.keys(privs).length > 0;

  const handleLogout = () => {
    localStorage.removeItem('moosic_token');
    localStorage.removeItem('moosic_user');
    navigate('/login');
  };

  const menuItems = [
    { key: '/', icon: <HomeOutlined />, label: 'Home' },
    { key: '/artists', icon: <CustomerServiceOutlined />, label: 'Artists' },
    { key: '/albums', icon: <PlayCircleOutlined />, label: 'Albums' },
    { key: '/songs', icon: <UnorderedListOutlined />, label: 'Songs' },
    { key: '/playlists', icon: <UnorderedListOutlined />, label: 'Playlists' },
    { key: '/search', icon: <SearchOutlined />, label: 'Search' },
    { key: '/bookmarks', icon: <BookOutlined />, label: 'Bookmarks' },
    { key: '/shares', icon: <ShareAltOutlined />, label: 'Shares' },
    { key: '/history', icon: <ClockCircleOutlined />, label: 'History' },
    ...(isAdmin ? [{ key: '/admin', icon: <SettingOutlined />, label: 'Admin' }] : []),
  ];

  return (
    <Sider width={220} style={{ display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '20px 16px 12px', textAlign: 'center' }}>
        <Text strong style={{ fontSize: 20, color: '#e94560' }}>🎧 Moosic</Text>
      </div>
      <Menu
        mode="inline"
        selectedKeys={[location.pathname]}
        items={menuItems}
        onClick={({ key }) => navigate(key)}
        style={{ flex: 1, borderRight: 0 }}
        theme="dark"
      />
      <div style={{ padding: '12px 16px', borderTop: '1px solid #ffffff15' }}>
        <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 8 }}>
          {user?.username}
        </Text>
        <Button type="text" icon={<LogoutOutlined />} onClick={handleLogout} block style={{ color: '#ffffffaa' }}>
          Logout
        </Button>
      </div>
    </Sider>
  );
}
