import React, { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/router';
import {
  Layout,
  Menu,
  Breadcrumb,
  Form,
  Button,
  Row,
  Col,
  Dropdown,
  Input,
  message,
  Card,
} from 'antd';
import { UserOutlined, SearchOutlined } from '@ant-design/icons';
import '../../node_modules/antd/dist/antd.css';
const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const GlobalHeader: React.FC = () => {
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  const router = useRouter();
  const site = router.pathname.split('/')[1];
  const menu = (
    <Menu
      id='nav'
      key='nav'
      theme='dark'
      mode={menuMode}
      defaultSelectedKeys={['home']}
      selectedKeys={[site]}
    >
      <Menu.Item key='home'>
        <Link href='/home'>首页</Link>
      </Menu.Item>
      <Menu.Item key='message'>
        <Link href='/message'>消息</Link>
      </Menu.Item>
      <Menu.Item key='trending'>
        <Link href='/trending'>热榜</Link>
      </Menu.Item>
      <Menu.Item key='searchpage'>
        <Link href='/searchpage'>搜索</Link>
      </Menu.Item>
    </Menu>
  );

  const UserMenu = (
    <Menu>
      <Menu.Item>
        <Link href='/profile'>个人信息</Link>
      </Menu.Item>
      <Menu.Divider />
      <Menu.Item
        onClick={() => {
          localStorage.removeItem('token');
          window.location.reload();
        }}
      >
        退出
      </Menu.Item>
    </Menu>
  );

  return (
    <Header>
      <Row>
        <div className='logo' />
        <Col offset={2}>{menu}</Col>
        <Col offset={16} span={1}>
          <Dropdown overlay={UserMenu} placement='bottomCenter'>
            <Button icon={<UserOutlined />} />
          </Dropdown>
        </Col>
      </Row>
    </Header>
  );
};

export default GlobalHeader;
