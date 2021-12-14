import type { NextPage, GetStaticProps } from 'next';
import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/router';
import { Layout, Breadcrumb, message, Card } from 'antd';
import PostList from '../components/post-list';
import GlobalHeader from '../components/header/header';
import '../node_modules/antd/dist/antd.css';
import axios, { AxiosError } from 'axios';

const { Header, Content, Footer } = Layout;

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const Home: NextPage = () => {
  const router = useRouter();
  const [menuMode, setMenuMode] = useState<'inline' | 'horizontal'>(
    'horizontal'
  );
  const [postList, setPostList] = useState([]);
  const [page, setPage] = useState(1);
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

  return (
    <Layout className='layout'>
      <Header>
        <title>首页</title>
        <GlobalHeader />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Breadcrumb style={{ margin: '16px 0' }}>
          <Breadcrumb.Item>分区1</Breadcrumb.Item>
          <Breadcrumb.Item>分区2</Breadcrumb.Item>
          <Breadcrumb.Item>分区3</Breadcrumb.Item>
        </Breadcrumb>
        <Card>
          <PostList listData={postList} setPage={setPage} />
        </Card>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default Home;
