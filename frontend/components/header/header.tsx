import React, { useState } from 'react';
import Link from 'next/link';
import type { NextPage } from 'next';
import { useRouter } from 'next/router';
import { Menu, Button, Row, Col, Dropdown, Input } from 'antd';
import { UserOutlined, PlusCircleOutlined } from '@ant-design/icons';
import '../../node_modules/antd/dist/antd.css';

const GlobalHeader: NextPage = () => {
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
      <Menu.Item key='create'>
        <Link href='/create'>发帖</Link>
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
  const CreateMenu = (
    <Menu>
      <Menu.Item>
        <Link href='/create'>发表帖子</Link>
      </Menu.Item>
    </Menu>
  );

  return (
    <Row>
      <div className='logo' />
      <Col offset={2}>{menu}</Col>
      <Col offset={14}>
        <Dropdown overlay={UserMenu} placement='bottomCenter'>
          <Button icon={<UserOutlined />} />
        </Dropdown>
      </Col>
      <Col>
        <Dropdown overlay={CreateMenu} placement='bottomCenter'>
          <Button icon={<PlusCircleOutlined />} style={{ margin: '10px' }} />
        </Dropdown>
      </Col>
    </Row>
  );
};

export default GlobalHeader;
