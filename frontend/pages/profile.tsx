import { ColumnsType } from 'antd/es/table';
import type { NextPage } from 'next';
import { useRouter } from 'next/router';
import axios, { AxiosError } from 'axios';
import PostList from '../components/post-list';
import { Layout, Table, Badge, message, Card } from 'antd';
import React, { useState, useEffect } from 'react';
import 'antd/dist/antd.css';
import '../styles/profile.module.css';
import GlobalHeader from '../components/header/header';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';
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
  const [postList, setPostList] = useState([]);

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
        setPostList(postlist);
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
          `${process.env.NEXT_PUBLIC_BASEURL}/users/burrow`
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

  return (
    <Layout className='layout'>
      <Header>
        <title>我的主页</title>
        <GlobalHeader />
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
          <PostList listData={postList} setPage={setPage} />
        </Card>
      </Content>
    </Layout>
  );
};

export default UserPage;
