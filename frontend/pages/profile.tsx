import { ColumnsType } from 'antd/es/table';
import type { NextPage } from 'next';
import Link from 'next/link';
import { useRouter } from 'next/router';
import { UserOutlined } from '@ant-design/icons';
import axios, { AxiosError } from 'axios';
import PostList from '../components/post-list';
import {
  Layout,
  Table,
  Menu,
  Badge,
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
import React, { useState, useEffect } from 'react';
import 'antd/dist/antd.css';
import '../styles/profile.module.css';

const { Header, Content, Footer } = Layout;
interface User {
  key: number;
  name: string;
  description?: string;
}

interface BurrowInfo {
  id: string;
  description: string;
  title: string;
}

interface MyBurrowInfo extends BurrowInfo {
  post_num: number;
}

interface FollowedBurrowInfo extends BurrowInfo {
  update: boolean;
}

interface PostInfo {}
const followedBurrowColumns: ColumnsType<FollowedBurrowInfo> = [
  {
    key: 'id',
    title: '洞号',
    dataIndex: 'id',
    render: (text, record, index) => {
      console.log(text, record.update);
      return record.update ? <Badge dot>{text}</Badge> : <Badge>{text}</Badge>;
    },
  },
  {
    key: 'title',
    title: '名称',
    dataIndex: 'title',
  },
  {
    key: 'description',
    title: '描述',
    dataIndex: 'description',
  },
];

const myBurrowColumns: ColumnsType<MyBurrowInfo> = [
  {
    key: 'id',
    title: '洞号',
    dataIndex: 'id',
  },
  {
    key: 'title',
    title: '名称',
    dataIndex: 'title',
  },
  {
    key: 'description',
    title: '描述',
    dataIndex: 'description',
  },
  {
    key: 'post_num',
    title: '帖子数',
    dataIndex: 'post_num',
  },
];
const UserPage: NextPage = () => {
  const router = useRouter();
  const [burrowList, setBurrowList] = useState([]);
  const [followedList, setFollowedList] = useState([]);
  const [page, setPage] = useState(1);
  const [postNum, setPostNum] = useState(1);
  const [postList, setPostList] = useState([]);
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  // 获取关注的帖子
  useEffect(() => {
    const fetchPostList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/content/list?page=${page - 1}`,
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
  // 获取我的地洞
  useEffect(() => {
    const fetchBurrowList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/burrom`
        );
        const burrowlist = res.data;
        setBurrowList(burrowlist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchBurrowList();
  }, [router]);
  // 获取收藏的洞
  useEffect(() => {
    const fetchFollowedList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/follow`
        );
        const followedlist = res.data;
        setFollowedList(followedlist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchFollowedList();
  }, [router]);
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
      <Content>
        <Card title='我的地洞'>
          <Table<MyBurrowInfo>
            columns={myBurrowColumns}
            dataSource={burrowList}
            rowKey='id'
          />
        </Card>
        <Card title='收藏的洞'>
          <Table<FollowedBurrowInfo>
            columns={followedBurrowColumns}
            dataSource={followedList}
            rowKey='id'
          />
        </Card>
        <Card title='收藏的帖子'>
          <PostList listData={postList} postNum={postNum} setPage={setPage} />
        </Card>
      </Content>
    </Layout>
  );
};

export default UserPage;
