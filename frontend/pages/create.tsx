import React, { useState, useEffect } from 'react';
import { Button, Card, Form, Input, Layout, message, Select } from 'antd';
import TextArea from 'antd/lib/input/TextArea';
import axios, { AxiosError } from 'axios';
import { NextPage } from 'next';
import GlobalHeader from '../components/header/header';
import { useRouter } from 'next/router';
import { Markdown } from '../components';
import UploadImage from '../components/storage/image-upload';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const Create: NextPage = () => {
  const { Header, Content, Footer } = Layout;
  const { Option } = Select;
  const router = useRouter();
  const [bidList, setBidList] = useState([]);
  const [content, setContent] = useState('');
  const [mode, setMode] = useState<'view' | 'edit'>('edit');
  const [newURL, setNewURL] = useState('');
  const toOption = (bidList: number[]) => {
    const bidOptionList = [];
    for (let i = 0; i < bidList.length; i++) {
      bidOptionList.push(
        <Option key={bidList[i].toString()} value={bidList[i]}>
          {'#' + bidList[i].toString() + ' 洞主'}
        </Option>
      );
    }
    return bidOptionList;
  };
  useEffect(() => {
    const fetchBid = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/valid-burrows`
        );
        const bidlist = res.data;
        setBidList(bidlist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchBid();
  }, [router]);

  useEffect(() => {
    const newContent =
      content +
      `![${newURL.slice(0, -5)}](${
        process.env.NEXT_PUBLIC_BASEURL
      }/storage/images/${newURL})`;
    setContent(newContent);
  }, [newURL]);

  const handleOnChange = (text: string) => {
    setContent(text);
  };

  const onFinish = async (values: any) => {
    const data = {
      ...values,
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/posts`,
        { ...data },
        { headers: { 'Content-Type': 'application/json' } }
      );
      message.success('发帖成功');
      window.location.reload();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status == 400) {
        message.error('格式不规范！');
      } else if (err.response?.status == 403) {
        message.error('用户被封禁或地洞不存在！');
      } else if (err.response?.status == 500) {
        message.error('服务器错误！');
      } else message.error('未知错误！');
    }
  };

  return (
    <Layout className='layout'>
      <Header>
        <title>发帖</title>
        <GlobalHeader />
      </Header>
      <Content style={{ padding: '0 50px' }}>
        <Card>
          <Form
            labelCol={{ span: 5 }}
            wrapperCol={{ span: 14 }}
            layout='horizontal'
            onFinish={onFinish}
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
              rules={[{ required: true, message: '帖子第一层不能为空' }]}
            >
              <Markdown
                content={content}
                mode={mode}
                editorStyle={{ height: '500px' }}
                onChange={handleOnChange}
              />
            </Form.Item>
            <Form.Item label='上传图片'>
              <UploadImage setNewURL={setNewURL} />
            </Form.Item>
            <Form.Item label='详情'>
              <Form.Item
                name='burrow_id'
                rules={[
                  { required: true, message: '请选择要以哪个洞主的身份发帖' },
                ]}
                style={{
                  display: 'inline-block',
                  width: 'calc(50% - 8px)',
                }}
                label='发帖人身份'
              >
                <Select placeholder='洞号'>{toOption(bidList)}</Select>
              </Form.Item>
              <Form.Item
                name='section'
                rules={[{ required: true, message: '请选择分区' }]}
                style={{
                  display: 'inline-block',
                  width: 'calc(50% - 8px)',
                  margin: '0 8px',
                }}
                label='贴子分区'
              >
                <Select
                  mode='multiple'
                  allowClear
                  style={{ width: '100%' }}
                  placeholder='分区(1-3个)'
                  maxTagCount={3}
                >
                  <Option value='Life'>日常生活</Option>
                  <Option value='Learning'>学习科研</Option>
                  <Option value='Entertainment'>休闲娱乐</Option>
                  <Option value='NSFW'>NSFW</Option>
                </Select>
              </Form.Item>
            </Form.Item>
            <Form.Item label='Tag' name='tag'>
              <Select
                mode='tags'
                allowClear
                style={{ width: '100%' }}
                placeholder='自定义Tag(0-10个)'
              />
            </Form.Item>
            <Form.Item wrapperCol={{ offset: 11, span: 16 }}>
              <Button type='primary' htmlType='submit'>
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
export default Create;
