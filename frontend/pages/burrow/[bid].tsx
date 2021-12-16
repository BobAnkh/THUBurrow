import React, { useState, useEffect } from 'react';
import type { NextPage } from 'next';
import {
  StarTwoTone,
  LikeTwoTone,
  UserOutlined,
  PlusCircleOutlined,
} from '@ant-design/icons';
import Link from 'next/link';
import styles from './burrow.module.css';
import { TextLoop } from 'react-text-loop-next';
import {
  Alert,
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
  Col,
  Dropdown,
  Row,
} from 'antd';
import { MessageOutlined, LikeOutlined, StarOutlined } from '@ant-design/icons';
import { useRouter } from 'next/router';
import 'antd/dist/antd.css';
import axios, { AxiosError } from 'axios';
import { createRouteLoader } from 'next/dist/client/route-loader';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const IconText = (props: any) => (
  <Space>
    {React.createElement(props.icon)}
    {props.text}
  </Space>
);

function showtag1(tag: string, index: number) {
  return <Tag key={index}>{tag}</Tag>;
}
const showtag = (value: Array<string>) => {
  return value.map(showtag1);
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
  const [isAlive, setIsAlive] = useState(true);

  const [editing, setEditing] = useState(false);
  const [descriptionTemp, setDescriptionTemp] = useState('');

  const [changeLike, setChangeLike] = useState(initialchange1);
  const [changeCol, setChangeCol] = useState(initialchange2);
  const [likeNum, setLikeNum] = useState(initialnum1);
  const [colNum, setColNum] = useState(initialnum2);

  const router = useRouter();
  const { bid } = router.query;
  const site = router.pathname.split('/')[1];
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );

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
        setIsAlive(postlist.isAlive);
      };
      fetchListData();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status === 400) {
        message.info('请先登录!');
        router.push('/login');
      } else if (err.response?.status === 500) {
        message.info('服务器错误!');
        router.push('/404');
      }
    }
  }, [router, page]);

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

  const EditIntro = () => {
    setEditing(true);
  };

  const ConfirmEdit = () => {
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
        <title>{`# ${bid} 地洞`}</title>
        <Row>
          <div className='logo' />
          <Col offset={2}>{menu}</Col>
          <Col offset={14}>
            <Dropdown overlay={UserMenu} placement='bottomCenter'>
              <Button icon={<UserOutlined />} />
            </Dropdown>
          </Col>
          <Col>
            <Dropdown
              overlay={CreateMenu}
              placement='bottomCenter'
              disabled={isAlive ? false : true}
            >
              <Button
                icon={<PlusCircleOutlined />}
                style={{ margin: '10px' }}
              />
            </Dropdown>
          </Col>
        </Row>
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
                <Form style={{ display: isAlive ? 'none' : 'block' }}>
                  <Alert
                    banner
                    type='warning'
                    showIcon
                    closeText='Close Now'
                    message={
                      <TextLoop mask>
                        <div>该洞已废弃</div>
                        <div>仅支持浏览帖子</div>
                        <div>您无法发表新帖</div>
                      </TextLoop>
                    }
                  />
                </Form>
                <div
                  style={{
                    margin: '20px 0px 0px 0px',
                    color: isAlive ? 'black' : 'grey',
                  }}
                >
                  # {bid}&emsp;{burrowTitle}
                </div>
              </h2>
              <div className={styles.Descript}>
                <h3 className={styles.BriefIntro}>简介:</h3>
                <Button
                  type='primary'
                  shape='round'
                  style={{
                    float: 'right',
                    display: isHost && !editing && isAlive ? 'block' : 'none',
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
                    title={
                      <a href={`../post/${item.post_id}`}>
                        {item.title}&emsp;
                        <Tag color='yellow'>{item.section}</Tag>
                      </a>
                    }
                  />
                  {showtag(item.tag)}
                </List.Item>
              )}
            />
          </Card>
        </div>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default Burrow;
