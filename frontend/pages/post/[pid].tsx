import type { NextPage } from 'next';
import React, { useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import {
  Layout,
  Breadcrumb,
  Form,
  Button,
  Input,
  message,
  Card,
  Select,
} from 'antd';
import {
  LikeOutlined,
  LikeTwoTone,
  StarOutlined,
  StarTwoTone,
} from '@ant-design/icons';
import ReplyList from '../../components/reply-list';
import '../../node_modules/antd/dist/antd.css';
import axios, { AxiosError } from 'axios';
import GlobalHeader from '../../components/header/header';
import Title from 'antd/lib/typography/Title';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;
const { Option } = Select;

const PostDetial: NextPage = () => {
  const router = useRouter();
  const { pid } = router.query;
  const pid_ = Number(pid);
  const [page, setPage] = useState(1);
  const [bid, setBid] = useState(1);
  const [replyList, setReplyList] = useState();
  const [postLen, setPostLen] = useState(1);
  const [title, setTitle] = useState('');
  const [bidList, setBidList] = useState([]);
  const [like, setLike] = useState(false);
  const [collection, setCollection] = useState(false);
  const initialchange1 = false;
  const initialchange2 = false;
  const [changeLike, setChangeLike] = useState(initialchange1);
  const [changeCol, setChangeCol] = useState(initialchange2);

  useEffect(() => {
    const fetchReplyList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/content/${pid}?page=${page - 1}`,
          {
            headers: { 'Content-Type': 'application/json' },
          }
        );
        const replylist = await res.data.post_page.reply_page;
        setReplyList(res.data.post_page.reply_page);
        setBid(res.data.post_page.post_desc.burrow_id);
        setTitle(res.data.post_page.post_desc.title);
        setLike(res.data.post_page.like);
        setCollection(res.data.post_page.collection);
        setPostLen(res.data.post_page.post_desc.post_len);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    const fetchBid = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/valid-burrows`
        );
        const bidlist = await res.data;
        setBidList(bidlist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchReplyList();
    fetchBid();
  }, [page, pid, router]);
  const toOption = (bidList: number[], bid: number) => {
    const bidOptionList = [];
    for (let i = 0; i < bidList.length; i++) {
      if (bid === bidList[i]) {
        bidOptionList.push(
          <Option key={bidList[i].toString()} value={bidList[i]}>
            {'#' + bidList[i].toString() + ' 洞主 (发帖人)'}
          </Option>
        );
      } else {
        bidOptionList.push(
          <Option key={bidList[i].toString()} value={bidList[i]}>
            {'#' + bidList[i].toString() + ' 洞主'}
          </Option>
        );
      }
    }
    return bidOptionList;
  };
  const clickCol = async (pid: number, activate: Boolean) => {
    const newChangeCol: boolean = !changeCol;
    setChangeCol(newChangeCol);
    try {
      if (activate) {
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateCollection: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
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

  const clickLike = async (pid: number, activate: Boolean) => {
    const newChangeLike: boolean = !changeLike;
    setChangeLike(newChangeLike);
    try {
      if (activate) {
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateLike: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
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

  const onFinish = async (values: any) => {
    const data = {
      ...values,
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/reply`,
        { ...data, post_id: pid },
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

  return (
    <Layout className='layout'>
      <Header>
        <GlobalHeader />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Card
          title={
            <>
              <Button
                icon={
                  (!like && changeLike) || (like && !changeLike) ? (
                    <LikeTwoTone twoToneColor='#8A2BE2' />
                  ) : (
                    <LikeOutlined />
                  )
                }
                onClick={() => {
                  clickLike(
                    pid_,
                    (like && changeLike) || (!like && !changeLike)
                  );
                }}
                style={{ float: 'right', margin: '10px' }}
              >
                {' ' + '点赞' + ' '}
              </Button>
              <Button
                icon={
                  (!collection && changeCol) || (collection && !changeCol) ? (
                    <StarTwoTone twoToneColor='#FFD700' />
                  ) : (
                    <StarOutlined />
                  )
                }
                onClick={() => {
                  clickCol(
                    pid_,
                    (collection && changeCol) || (!collection && !changeCol)
                  );
                }}
                style={{ float: 'right', margin: '10px' }}
              >
                {' ' + '收藏' + ' '}
              </Button>
              <Title level={3} style={{ float: 'left', margin: '10px' }}>
                {title}
              </Title>
            </>
          }
        >
          <ReplyList
            listData={replyList}
            setPage={setPage}
            userBid={bidList}
            totalNum={postLen}
          />
          <Form
            labelCol={{ span: 5 }}
            wrapperCol={{ span: 14 }}
            layout='horizontal'
            onFinish={onFinish}
            style={{ padding: '20px' }}
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
            <Form.Item
              label='身份'
              name='burrow_id'
              rules={[
                { required: true, message: '请选择要以哪个洞主的身份回复' },
              ]}
            >
              <Select placeholder='洞号'>{toOption(bidList, bid)}</Select>
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
