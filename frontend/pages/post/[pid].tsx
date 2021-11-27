import type { NextPage, GetStaticProps } from 'next';
import React, { useEffect, useState } from 'react';
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
import ReplyList from '../../components/reply-list';
import '../../node_modules/antd/dist/antd.css';
import axios, { AxiosError } from 'axios';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const PostDetial: NextPage = () => {
  const router = useRouter();
  const { pid } = router.query;
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  const [page, setPage] = useState(1);
  const [bid, setBid] = useState(1);
  const [replyList, setReplyList] = useState();
  const [postLen, setPostLen] = useState(0);
  const [title, setTitle] = useState('test');
  useEffect(() => {
    const fetchReplyList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/content/${pid}`,
          {
            headers: { 'Content-Type': 'application/json' },
          }
        );
        const replylist = res.data.post_page.reply_page;
        setBid(res.data.post_page.post_desc.burrow_id);
        setPostLen(res.data.post_page.post_desc.post_len);
        setTitle(res.data.post_page.post_desc.title);
        setReplyList(replylist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
        console.error(error);
      }
    };
    fetchReplyList();
  }, [pid, router]);

  const onFinish = async (values: any) => {
    const data = {
      ...values,
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/reply`,
        { ...data, burrow_id: bid, post_id: pid },
        { headers: { 'Content-Type': 'application/json' } }
      );
      const json = await res.data;
      if (json.success === false) {
        message.error('回复失败');
      } else {
        message.success('回复成功');
        window.location.reload();
      }
    } catch (e) {
      message.error('回复失败');
    }
  };

  const onFinishFailed = (errorInfo: any) => {
    message.error(errorInfo);
  };

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
      <Menu.Item key='search'>
        <Link href='/search/searchpage'>搜索</Link>
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
        <title>{title}</title>
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
        <Card title={title}>
          <ReplyList listData={replyList} postLen={postLen} setPage={setPage} />
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
              label='回复内容'
              name='content'
              rules={[{ required: true, message: '回复不能为空' }]}
            >
              <TextArea
                rows={4}
                placeholder={'友善的沟通是高质量交流的第一步~'}
              />
            </Form.Item>
            <Form.Item wrapperCol={{ offset: 11, span: 16 }}>
              <Button
                type='primary'
                htmlType='submit'
                style={{ margin: '16px 0' }}
              >
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

export default PostDetial;
