import type { NextPage, GetStaticProps } from 'next';
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
import { UserOutlined } from '@ant-design/icons';
import moment from 'moment';
import PostList from '../components/post-list';
import '../node_modules/antd/dist/antd.css';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const onFinish = async (values: any) => {
  const time = moment().format('YYYY-MM-DD HH:mm:ss');
  const data = {
    ...values,
    author: 'yonghu',
    create_time: time,
    modified_time: '',
    anomymous: false,
    section: 'daily',
    tag1: 'zai',
    tag2: 'zuo',
    tag3: 'le',
  };
  try {
    const res = await fetch('http://127.0.0.1:4523/mock/435762/content', {
      method: 'POST',
      body: JSON.stringify(data),
    });
    const json = await res.json();
    if (json.success === false) {
      message.error('发帖失败');
    } else {
      message.success('发帖成功');
      window.location.href = './';
    }
  } catch (e) {
    message.error('发帖失败');
  }
};

const onFinishFailed = (errorInfo: any) => {
  console.log('Failed:', errorInfo);
};

const Home = (props: any) => {
  const listData: any = props.data;
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
    <Layout className='layout'>
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
      <Content style={{ padding: '0 50px' }}>
        <Breadcrumb style={{ margin: '16px 0' }}>
          <Breadcrumb.Item>Home</Breadcrumb.Item>
          <Breadcrumb.Item>List</Breadcrumb.Item>
          <Breadcrumb.Item>App</Breadcrumb.Item>
        </Breadcrumb>
        <Card>
          <PostList listData={listData} />
          <Form
            labelCol={{ span: 5 }}
            wrapperCol={{ span: 14 }}
            layout='horizontal'
            onFinish={onFinish}
            onFinishFailed={onFinishFailed}
            style={{
              margin: 'auto',
              padding: '10px',
            }}
          >
            <Form.Item
              label='标题'
              name='title'
              rules={[{ required: true, message: '标题不能为空' }]}
            >
              <Input placeholder='请输入标题' />
            </Form.Item>
            <Form.Item
              label='内容'
              name='content'
              rules={[{ required: true, message: '第一层洞不能为空' }]}
            >
              <TextArea rows={4} />
            </Form.Item>
            <Form.Item wrapperCol={{ offset: 11, span: 16 }}>
              <Button type='primary' htmlType='submit'>
                发布
              </Button>
            </Form.Item>
          </Form>
        </Card>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};
export async function getStaticProps() {
  const res = await fetch('http://127.0.0.1:4523/mock/435762/contents', {
    method: 'GET',
  });
  if (res.status === 401) {
    message.info('请先登录！');
    return {
      redirect: {
        destination: '/login',
        permanent: false,
      },
    };
  } else {
    const data = await res.json();
    if (!data) {
      return {
        notFound: true,
      };
    }

    return {
      props: { data }, // will be passed to the page component as props
    };
  }
}

export default Home;
