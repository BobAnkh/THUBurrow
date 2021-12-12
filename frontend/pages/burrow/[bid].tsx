import React, { useState, useEffect } from 'react';
import type { NextPage, GetStaticProps } from 'next';
import { StarTwoTone, LikeTwoTone } from '@ant-design/icons';
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
  Tag,
} from 'antd';
import Link from 'next/link';
import { MessageOutlined, LikeOutlined, StarOutlined } from '@ant-design/icons';
import { useRouter } from 'next/router';
import moment from 'moment';
import 'antd/dist/antd.css';
import axios, { AxiosError } from 'axios';
import { TYPES } from '@babel/types';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const IconText = (props: any) => (
  <Space>
    {React.createElement(props.icon)}
    {props.text}
  </Space>
);

function showtag1(tag: string) {
    return <Tag>{tag}</Tag>;
  }
  const showtag = (value: Array<string>) => {
    return value.map(showtag1);
  };

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
      {
        ...data,
      },
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
  }
};

const onFinishFailed = (errorInfo: any) => {
  if (errorInfo.values.title == undefined) message.error('标题不能为空！');
  else message.error('内容不能为空！');
};

const Burrow: NextPage = () => {
  const initialchange1 = new Array(10).fill(false);
  const initialchange2 = new Array(10).fill(false);
  const initialnum1 = new Array(10).fill(0);
  const initialnum2 = new Array(10).fill(0);

  const [listData, setListData] = useState([]);
  const [description, setDescription] = useState('Welcome!');
  const [burrowTitle, setBurrowTitle] = useState(0);
  const [page, setPage] = useState(1);
  const [isHost, setIsHost] = useState(true);
  const [editing, setEditing] = useState(false);
  const [descriptionTemp, setDescriptionTemp] = useState('');
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  const [changeLike, setChangeLike] = useState(initialchange1);
  const [changeCol, setChangeCol] = useState(initialchange2);
  const [likeNum, setLikeNum] = useState(initialnum1);
  const [colNum, setColNum] = useState(initialnum2);

  const router = useRouter();
  const { bid } = router.query;
  const site = router.pathname.split('/')[1];

  useEffect(() => {
    try {
      const fetchListData = async () => {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/${bid}?page=${page - 1}`
        );
        const postlist = await res.data;
        setListData(postlist.posts);
        setDescription(postlist.description);
        setBurrowTitle(postlist.title);
        setIsHost(postlist.isHost);
      };
      fetchListData();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status === 400) {
        message.info('请先登录！');
        router.push('/login');
      } else if (err.response?.status === 500) {
        message.info('服务器错误！');
        router.push('/404');
      }
    }
  }, [router, page]);

  const EditIntro = () => {
    setEditing(true);
  };

  const ConfirmEdit = () => {
    console.log(descriptionTemp);
    setDescription(descriptionTemp);
    setEditing(false);
  };

  const CancelEdit = () => {
    setEditing(false);
  };

  const UpdateIntro = (event: any) => {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setDescriptionTemp(value);
    }
  };

  const clickCol = async (pid: number, activate: Boolean, index: number) => {
    let newChangeCol: boolean[] = changeCol;
    newChangeCol[index] = !changeCol[index];
    setChangeCol([...newChangeCol]);
    const newColNum = colNum;
    try {
      if (activate) {
        newColNum[index] = colNum[index] + 1;
        setColNum([...newColNum]);
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateCollection: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
        newColNum[index] = colNum[index] - 1;
        setColNum([...newColNum]);
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { DeactivateCollection: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      }
    } catch (e) {
      if (activate) {
        message.error('收藏失败');
      } else {
        message.error('取消收藏失败');
      }
    }
  };

  const clickLike = async (pid: number, activate: Boolean, index: number) => {
    let newChangeLike: boolean[] = changeLike;
    newChangeLike[index] = !changeLike[index];
    setChangeLike([...newChangeLike]);
    const newLikeNum = likeNum;
    try {
      if (activate) {
        newLikeNum[index] = likeNum[index] + 1;
        setLikeNum([...newLikeNum]);
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateLike: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
        newLikeNum[index] = likeNum[index] - 1;
        setLikeNum([...newLikeNum]);
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { deactivateLike: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      }
    } catch (e) {
      if (activate) {
        message.error('点赞失败');
      } else {
        message.error('取消点赞失败');
      }
    }
  };

  return (
    <Layout>
      <Header style={{ position: 'fixed', zIndex: 1, width: '100%' }}>
        <div className='logo' />
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
            <div>
              <h2>
                # {bid}&emsp;{burrowTitle}
              </h2>
              <div className={styles.Descript}>
                <h3 className={styles.BriefIntro}>简介:</h3>
                <Button
                  type='primary'
                  shape='round'
                  style={{
                    float: 'right',
                    display: isHost && !editing ? 'block' : 'none',
                  }}
                  onClick={EditIntro}
                >
                  编辑
                </Button>
                <div
                  style={{
                    paddingLeft: '35px',
                    display: editing ? 'none' : 'block',
                  }}
                >
                  {description}
                </div>
                <Form
                  style={{
                    paddingLeft: '35px',
                    display: editing ? 'block' : 'none',
                  }}
                >
                  <TextArea
                    autoSize={{ minRows: 2, maxRows: 6 }}
                    className={styles.EditText}
                    onChange={(event) => UpdateIntro(event)}
                  />
                  <Button
                    className={styles.Cancel}
                    onClick={CancelEdit}
                    shape='round'
                  >
                    取消
                  </Button>
                  <Button
                    className={styles.Confirm}
                    type='primary'
                    shape='round'
                    onClick={ConfirmEdit}
                  >
                    确认
                  </Button>
                </Form>
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
                    <Button
                      type='text'
                      icon={
                        (changeLike[index] && item.like) ||
                        (!changeLike[index] && !item.like) ? (
                          <LikeTwoTone twoToneColor='#8A2BE2' />
                        ) : (
                          <LikeOutlined />
                        )
                      }
                      key='list-vertical-like-o'
                      onClick={() => {
                        clickLike(
                          item.post_id,
                          (!changeLike[index] && item.like) ||
                            (changeLike[index] && !item.like),
                          index
                        );
                      }}
                      className={styles.ButtonLayout}
                    >
                      {' '}
                      {item.like_num + likeNum[index]}
                    </Button>,
                    <Button
                      type='text'
                      icon={
                        (!changeCol[index] && item.collection) ||
                        (changeCol[index] && !item.collection) ? (
                          <StarTwoTone twoToneColor='#FFD700' />
                        ) : (
                          <StarOutlined />
                        )
                      }
                      key='list-vertical-star-o'
                      onClick={() => {
                        clickCol(
                          item.post_id,
                          (changeCol[index] && item.collection) ||
                            (!changeCol[index] && !item.collection),
                          index
                        );
                      }}
                      className={styles.ButtonLayout}
                    >
                      {' '}
                      {item.collection_num + colNum[index]}
                    </Button>,
                    <IconText
                      icon={MessageOutlined}
                      text={item.post_len}
                      key='list-vertical-message'
                      className={styles.ButtonLayout}
                    />,
                  ]}
                >
                  <List.Item.Meta
                    title={<a href={`post/${item.post_id}`}>{item.title}&emsp;<Tag color="yellow">{item.section}</Tag></a>}
                  />
                  {showtag(item.tag)}
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
