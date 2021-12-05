import type { NextPage, GetStaticProps } from 'next';
import React, { useState, useEffect } from 'react';
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
import axios, { AxiosError } from 'axios';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

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
    const res = await axios.post(
      `${process.env.NEXT_PUBLIC_BASEURL}/content/post`,
      { ...data },
      { headers: { 'Content-Type': 'application/json' } }
    );
    const json = await res.data;
    if (json.error) {
      message.error('发帖失败');
    } else {
      message.success('发帖成功');
      window.location.reload();
    }
  } catch (e) {
    message.error('发帖失败');
    alert(e);
  }
};

const onFinishFailed = (errorInfo: any) => {
  message.error(errorInfo);
};

const Home: NextPage = () => {
  const router = useRouter();
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  const [postList, setPostList] = useState([]);
  const [page, setPage] = useState(1);
  const [postNum, setPostNum] = useState(0);
  useEffect(() => {
    const fetchPostList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/content/list?page=${page}`,
          {
            headers: { 'Content-Type': 'application/json' },
          }
        );
        const postlist = res.data.list_page.post_page;
        const postnum = res.data.list_page.post_num;
        setPostList(postlist);
        setPostNum(postnum);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchPostList();
  }, [page, router]);

  const site = router.pathname.split('/')[1];
  const menu = (
    <Menu
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
      <Menu.Item key='search'>
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
    <Layout className='layout'>
      <Header>
        <title>首页</title>
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
          <PostList listData={postList} postNum={postNum} setPage={setPage} />
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

export default Home;
