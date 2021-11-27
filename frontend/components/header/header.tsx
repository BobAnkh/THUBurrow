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
      <Menu.Item key='update'>
        <Link href='/update'>动态</Link>
      </Menu.Item>
      <Menu.Item key='setting'>
        <Link href='/setting'>设置</Link>
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
        <Col offset={12}>
          <Button icon={<SearchOutlined />} />
        </Col>
        <Col offset={1}>
          <Dropdown overlay={UserMenu} placement='bottomCenter'>
            <Button href='../search/searchPage' icon={<UserOutlined />} />
          </Dropdown>
        </Col>
      </Row>
    </Header>
  );
};

export default GlobalHeader;
