import type { NextPage } from 'next';
import React, { useState, useEffect } from 'react';
import { useRouter } from 'next/router';
import { Layout, message, Card } from 'antd';
import { PostColList } from '../components/post-list';
import '../node_modules/antd/dist/antd.css';
import axios, { AxiosError } from 'axios';
import GlobalHeader from '../components/header/header';

const { Header, Content, Footer } = Layout;
axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const Trending: NextPage = () => {
  const router = useRouter();
  const [postList, setPostList] = useState([]);
  const [page, setPage] = useState(1);

  useEffect(() => {
    const fetchPostList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/trending`,
          {
            headers: { 'Content-Type': 'application/json' },
          }
        );
        const postlist = JSON.parse(res.data);
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

  return (
    <Layout className='layout'>
      <Header>
        <title>热榜</title>
        <GlobalHeader />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Card>
          <PostColList listData={postList} setPage={setPage} totalNum={50} />
        </Card>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default Trending;
