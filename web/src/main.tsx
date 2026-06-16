import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { ConfigProvider, App, theme } from 'antd';
import AppRouter from './App';
import './index.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <ConfigProvider
        theme={{
          algorithm: theme.darkAlgorithm,
          token: {
            colorPrimary: '#e94560',
            borderRadius: 8,
            colorBgContainer: '#1a1a2e',
            colorBgElevated: '#1a1a2e',
          },
        }}
      >
        <App>
          <AppRouter />
        </App>
      </ConfigProvider>
    </BrowserRouter>
  </StrictMode>,
);
