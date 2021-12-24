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

// const mockData =
// '[{"post_id":2,"title":"试试看发图","burrow_id":1,"section":["Entertainment"],"tag":["tag"],"create_time":"2021-12-24T08:35:33.902354+00:00","update_time":"2021-12-24T09:14:42.673801+00:00","post_state":0,"post_type":0,"like_num":2,"collection_num":1,"post_len":4},{"post_id":6,"title":"洞主#1请问我应该如何关注您！","burrow_id":7,"section":["Life"],"tag":["一个提问"],"create_time":"2021-12-24T09:18:19.157143+00:00","update_time":"2021-12-24T09:24:58.625869+00:00","post_state":0,"post_type":0,"like_num":1,"collection_num":1,"post_len":7},{"post_id":1,"title":"我来组成头部","burrow_id":1,"section":["Life"],"tag":["tag"],"create_time":"2021-12-24T08:21:42.633509+00:00","update_time":"2021-12-24T08:54:38.624938+00:00","post_state":0,"post_type":0,"like_num":2,"collection_num":2,"post_len":1},{"post_id":7,"title":"如果贴主连续发两条回复，点第二个回复的编辑，会编辑到第一个回复（编号#0的那个）","burrow_id":7,"section":["Learning","Life"],"tag":["一个bug"],"create_time":"2021-12-24T09:22:16.743817+00:00","update_time":"2021-12-24T09:29:07.344798+00:00","post_state":0,"post_type":0,"like_num":1,"collection_num":0,"post_len":6},{"post_id":3,"title":"进行一个帖的测","burrow_id":7,"section":["Entertainment","Learning","Life"],"tag":["一个测试"],"create_time":"2021-12-24T09:15:56.375353+00:00","update_time":"2021-12-24T09:16:50.466444+00:00","post_state":0,"post_type":0,"like_num":1,"collection_num":1,"post_len":1},{"post_id":9,"title":"前端的朋友们，section的学习和生活都会显示成生活，看图↓","burrow_id":7,"section":["Entertainment","Learning","Life"],"tag":["一个bug"],"create_time":"2021-12-24T09:33:28.195668+00:00","update_time":"2021-12-24T09:43:42.005839+00:00","post_state":0,"post_type":0,"like_num":0,"collection_num":0,"post_len":4},{"post_id":8,"title":"或许可以多发点帖子看看翻页的bug修好了嘛（？）各位uu们辛苦了！","burrow_id":7,"section":["Life"],"tag":["一个提问"],"create_time":"2021-12-24T09:31:19.238423+00:00","update_time":"2021-12-24T09:44:11.440956+00:00","post_state":0,"post_type":0,"like_num":0,"collection_num":0,"post_len":3},{"post_id":11,"title":"section测试","burrow_id":1,"section":["Learning","Life","NSFW"],"tag":["bug"],"create_time":"2021-12-24T09:41:56.984746+00:00","update_time":"2021-12-24T09:43:02.335996+00:00","post_state":0,"post_type":0,"like_num":0,"collection_num":0,"post_len":2},{"post_id":4,"title":"删帖测试","burrow_id":2,"section":["Learning"],"tag":["1111"],"create_time":"2021-12-24T09:16:46.855997+00:00","update_time":"2021-12-24T09:25:46.685144+00:00","post_state":0,"post_type":0,"like_num":0,"collection_num":0,"post_len":2},{"post_id":10,"title":"“显示洞”界面里帖子的section和“帖子详情”里帖子的section不一致，看图↓","burrow_id":7,"section":["Learning","Life"],"tag":["一个提问"],"create_time":"2021-12-24T09:37:45.892228+00:00","update_time":"2021-12-24T09:38:29.872648+00:00","post_state":0,"post_type":0,"like_num":0,"collection_num":0,"post_len":2},{"post_id":5,"title":"dd","burrow_id":1,"section":["Life"],"tag":["tag"],"create_time":"2021-12-24T09:18:00.297429+00:00","update_time":"2021-12-24T09:23:43.478876+00:00","post_state":0,"post_type":0,"like_num":0,"collection_num":0,"post_len":2}]';
const Trending: NextPage = () => {
  const router = useRouter();
  const [postList, setPostList] = useState([]);
  const [page, setPage] = useState(1);

  useEffect(() => {
    const fetchPostList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/trending`,
          // 'http://127.0.0.1:4523/mock/435762/trending',
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
