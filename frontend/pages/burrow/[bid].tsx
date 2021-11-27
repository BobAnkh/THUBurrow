import React, { useState, useEffect } from 'react';
import type { NextPage, GetStaticProps } from 'next';
import styles from './burrow.module.css';
import {
  Layout,
  Menu,
  Breadcrumb,
  List,
  Space,
  message,
  Form,
  Button,
  Input,
  Card,
} from 'antd';
import Link from 'next/link';
import { MessageOutlined, LikeOutlined, StarOutlined } from '@ant-design/icons';
import { useRouter } from 'next/router';
import moment from 'moment';
import 'antd/dist/antd.css';
import { TYPES } from '@babel/types';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const IconText = (props: any) => (
  <Space>
    {React.createElement(props.icon)}
    {props.text}
  </Space>
);

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
    const res = await fetch(`${process.env.NEXT_PUBLIC_BASEURL}/content/post`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
    const json = await res.json();
    if (json.success === false) {
      message.error('发帖失败');
    } else {
      message.success('发帖成功');
      window.location.reload();
    }
  } catch (e) {
    message.error('发帖失败');
  }
};

const onFinishFailed = (errorInfo: any) => {
  if (errorInfo.values.title == undefined) message.error('标题不能为空！');
  else message.error('内容不能为空！');
};

const Burrow: NextPage = () => {
  const [listData, setListData] = useState([]);
  const [description, setDescription] = useState('Welcome!');
  const [burrowTitle, setBurrowTitle] = useState(0);
  const [page, setPage] = useState(1);

  const router = useRouter();
  const { bid } = router.query;

  useEffect(() => {
    const fetchListData = async () => {
      console.log(`ad: ${process.env.NEXT_PUBLIC_BASEURL}/burrows/1`);
      const res = await fetch(`${process.env.NEXT_PUBLIC_BASEURL}/burrows/1`, {
        method: 'GET',
      });
      if (res.status === 401) {
        message.info('请先登录！');
        router.push('/login');
      } else {
        const postlist = await res.json();
        setListData(postlist.posts);
        setDescription(postlist.description);
        setBurrowTitle(postlist.title);
      }
    };
    fetchListData();
  }, [router, page]);

  return (
    <Layout>
      <Header style={{ position: 'fixed', zIndex: 1, width: '100%' }}>
        <div className='logo' />
        <Menu theme='dark' mode='horizontal'>
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
      </Header>
      <Content
        className='site-Layout'
        style={{ padding: '0 50px', marginTop: 64 }}
      >
        <Breadcrumb style={{ margin: '16px 0' }}>
          <Breadcrumb.Item>Home</Breadcrumb.Item>
          <Breadcrumb.Item>List</Breadcrumb.Item>
          <Breadcrumb.Item>App</Breadcrumb.Item>
        </Breadcrumb>

        <div
          className='site-layout-background'
          style={{ padding: 24, minHeight: 380 }}
        >
          <Card>
            <div className={styles.intro}>
              <h2>{`#${bid} ${burrowTitle}`}</h2>
              <div className={styles.Descript}>
                <h4>简介</h4>
                <div style={{ paddingLeft: '35px' }}>{description}</div>
              </div>
            </div>
            <List
              itemLayout='vertical'
              size='large'
              pagination={{
                onChange: (page) => {
                  setPage(page);
                },
                pageSize: 20,
              }}
              dataSource={listData}
              footer={
                <div>
                  <b>THUBurrow</b> footer part
                </div>
              }
              renderItem={(item: any, index: any) => (
                <List.Item
                  style={{
                    background: index % 2 === 0 ? '#f1f4f8' : '#FFFFFF',
                  }}
                  key={item.title}
                  actions={[
                    <IconText
                      icon={StarOutlined}
                      text={item.stars}
                      key='list-vertical-star-o'
                    />,
                    <IconText
                      icon={LikeOutlined}
                      text={item.likes}
                      key='list-vertical-like-o'
                    />,
                    <IconText
                      icon={MessageOutlined}
                      text={item.message}
                      key='list-vertical-message'
                    />,
                  ]}
                >
                  <List.Item.Meta
                    title={<a href={`/post/${item.post_id}`}>{item.title}</a>}
                  />
                  {item.content}
                </List.Item>
              )}
            />

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
        </div>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default Burrow;
