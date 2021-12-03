import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/router';
import { Layout, Menu, Button, Row, Col, Dropdown, Space, Popover } from 'antd';
import { UserOutlined, MenuOutlined } from '@ant-design/icons';
import '../../node_modules/antd/dist/antd.css';

import useWindowDimensions from '../../hooks/useWindowDimensions';

const Header: React.FC = () => {
  const { width: windowWidth } = useWindowDimensions();
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  const router = useRouter();
  const site = router.pathname.split('/')[1];

  useEffect(() => {
    if (windowWidth! < 767) {
      setMenuMode('inline');
    } else {
      setMenuMode('horizontal');
    }
  }, [windowWidth]);

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

  const userMenu = (
    <Menu>
      <Menu.Item>
        <Link href='/profile'>个人信息</Link>
      </Menu.Item>
      <Menu.Divider />
      <Menu.Item
        onClick={() => {
          window.location.reload();
        }}
      >
        退出
      </Menu.Item>
    </Menu>
  );

  return (
    <Layout.Header>
      <Row justify='space-between'>
        <Col xxl={4} xl={4} lg={7} md={7} sm={19} xs={19}>
          <Space size='large'>{/* TODO: logo */}</Space>
        </Col>
        <Col xxl={19} xl={19} lg={16} md={16} sm={4} xs={4}>
          {menuMode === 'inline' ? (
            <Popover placement='bottomRight' content={menu} trigger='click'>
              <Button icon={<MenuOutlined />} size='large' type='text' />
            </Popover>
          ) : (
            menu
          )}
        </Col>
        <Col span={1}>
          <Dropdown overlay={userMenu} placement='bottomCenter'>
            <Button icon={<UserOutlined />} />
          </Dropdown>
        </Col>
      </Row>
    </Layout.Header>
  );
};

export default Header;
