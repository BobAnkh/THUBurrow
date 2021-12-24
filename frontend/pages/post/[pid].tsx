import type { NextPage } from 'next';
import React, { useEffect, useState } from 'react';
import { useRouter } from 'next/router';
import {
  Layout,
  Form,
  Button,
  message,
  Card,
  Select,
  Tag,
  Popconfirm,
  Modal,
  Input,
} from 'antd';
import {
  LikeOutlined,
  LikeTwoTone,
  StarOutlined,
  StarTwoTone,
  EditOutlined,
  DeleteOutlined,
} from '@ant-design/icons';
import ReplyList from '../../components/reply-list';
import '../../node_modules/antd/dist/antd.css';
import axios, { AxiosError } from 'axios';
import GlobalHeader from '../../components/header/header';
import Title from 'antd/lib/typography/Title';
import { Markdown } from '../../components';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const { Header, Content, Footer } = Layout;
const { Option } = Select;

const PostDetial: NextPage = () => {
  const router = useRouter();
  const { pid } = router.query;
  const pid_ = Number(pid);
  const [page, setPage] = useState(1);
  const [bid, setBid] = useState(1);
  const [replyList, setReplyList] = useState();
  const [postLen, setPostLen] = useState(1);
  const [section, setSection] = useState([]);
  const [tag, setTag] = useState([]);
  const [title, setTitle] = useState('');
  const initialBidList: number[] = [];
  const [bidList, setBidList] = useState(initialBidList);
  const [like, setLike] = useState(false);
  const [collection, setCollection] = useState(false);
  const [edit, setEdit] = useState(false);
  const initialchange1 = false;
  const initialchange2 = false;
  const [changeLike, setChangeLike] = useState(initialchange1);
  const [changeCol, setChangeCol] = useState(initialchange2);
  const [replyContent, setReplyContent] = useState('');
  const [editContent, setEditContent] = useState('');
  const [mode, setMode] = useState<'view' | 'edit'>('edit');

  useEffect(() => {
    const fetchReplyList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/content/posts/${pid}?page=${
            page - 1
          }`,
          {
            headers: { 'Content-Type': 'application/json' },
          }
        );
        setReplyList(res.data.reply_page);
        setBid(res.data.post_desc.burrow_id);
        setTitle(res.data.post_desc.title);
        setLike(res.data.like);
        setCollection(res.data.collection);
        setPostLen(res.data.post_desc.post_len);
        setSection(res.data.post_desc.section);
        setTag(res.data.post_desc.tag);
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
  }, [page, router]);

  function showtag1(tag: string, index: number) {
    if (tag === '') return null;
    return <Tag key={index}>{tag}</Tag>;
  }
  function showsection1(tag: string, index: number) {
    return (
      <Tag key={index} color='blue'>
        {tag}
      </Tag>
    );
  }

  const showtag = (value: Array<string>) => {
    return (value || []).map(showtag1);
  };

  const showsection = (value: Array<string>) => {
    return value.map(showsection1);
  };

  const handleOnEditChange = (text: string) => {
    setEditContent(text);
  };

  const handleOnReplyChange = (text: string) => {
    setReplyContent(text);
  };

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
          { DeactivateLike: pid },
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

  const onDelete = async () => {
    try {
      axios.delete(`${process.env.NEXT_PUBLIC_BASEURL}/content/posts/${pid}`);
      message.success('删除成功！');
      window.location.reload();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status == 400) {
        message.error('用户不存在！');
      } else if (err.response?.status == 403) {
        message.error('用户被封禁或发帖超过2分钟，无法删除！');
      } else if (err.response?.status == 404) {
        message.error('帖子不存在！');
      } else if (err.response?.status == 500) {
        message.error('服务器错误！');
      }
    }
  };

  const handleChange = async (values: any) => {
    const data = {
      ...values,
    };
    try {
      const res = await axios.patch(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/posts`,
        { ...data, burrow_id: bid },
        { headers: { 'Content-Type': 'application/json' } }
      );

      message.success('修改成功！');
      window.location.reload();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status == 400) {
        message.error('格式不规范！');
      } else if (err.response?.status == 403) {
        message.error('用户被封禁或地洞不存在！');
      } else if (err.response?.status == 404) {
        message.error('帖子不存在！');
      } else if (err.response?.status == 500) {
        message.error('服务器错误！');
      } else message.error('未知错误！');
    }
  };

  const onFinish = async (values: any) => {
    const data = {
      ...values,
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/replies`,
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
        <title>{title}</title>
        <GlobalHeader />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Card
          title={
            <>
              {bidList.indexOf(bid) === -1 ? (
                <></>
              ) : (
                <>
                  <Popconfirm
                    placement='topRight'
                    title='确认删除该贴?'
                    onConfirm={onDelete}
                    okText='Yes'
                    cancelText='No'
                  >
                    <Button
                      icon={<DeleteOutlined />}
                      style={{ float: 'right', margin: '10px' }}
                      danger
                    >
                      {'' + '删除' + ''}
                    </Button>
                  </Popconfirm>
                  <Button
                    icon={<EditOutlined />}
                    style={{ float: 'right', margin: '10px' }}
                    onClick={() => setEdit(true)}
                  >
                    {'' + '编辑' + ''}
                  </Button>
                  <Modal
                    title='修改帖子'
                    visible={edit}
                    onCancel={() => setEdit(false)}
                    width='80%'
                    footer={null}
                  >
                    <Form
                      labelCol={{ span: 5 }}
                      wrapperCol={{ span: 14 }}
                      layout='horizontal'
                      onFinish={handleChange}
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
                        rules={[
                          { required: true, message: '帖子第一层不能为空' },
                        ]}
                      >
                        <Markdown
                          content={editContent}
                          mode={mode}
                          editorStyle={{ height: '500px' }}
                          onChange={handleOnEditChange}
                        />
                      </Form.Item>
                      <Form.Item
                        name='section'
                        rules={[{ required: true, message: '请选择分区' }]}
                        label='贴子分区'
                      >
                        <Select
                          mode='multiple'
                          allowClear
                          style={{ width: '100%' }}
                          placeholder='分区'
                        >
                          <Option value='Life'>日常生活</Option>
                          <Option value='Learning'>学习科研</Option>
                          <Option value='Entertainment'>休闲娱乐</Option>
                          <Option value='NSFW'>NSFW</Option>
                        </Select>
                      </Form.Item>
                      <Form.Item label='Tag' name='tag'>
                        <Select
                          mode='tags'
                          allowClear
                          style={{ width: '100%' }}
                          placeholder='Tag'
                        />
                      </Form.Item>
                      <Form.Item wrapperCol={{ offset: 11, span: 16 }}>
                        <Button type='primary' htmlType='submit'>
                          确认修改
                        </Button>
                      </Form.Item>
                    </Form>
                  </Modal>
                </>
              )}
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
          {showsection(section)}
          {showtag(tag)}
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
              label='内容'
              name='content'
              rules={[{ required: true, message: '回复不能为空！' }]}
            >
              <Markdown
                content={replyContent}
                mode={mode}
                editorStyle={{ height: '500px' }}
                onChange={handleOnReplyChange}
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
                发表回复
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
