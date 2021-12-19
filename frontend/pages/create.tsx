import React, { useState, useEffect } from 'react';
import { Button, Card, Form, Input, Layout, message, Select } from 'antd';
import TextArea from 'antd/lib/input/TextArea';
import axios, { AxiosError } from 'axios';
import { NextPage } from 'next';
import GlobalHeader from '../components/header/header';
import { useRouter } from 'next/router';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const Create: NextPage = () => {
  const { Header, Content, Footer } = Layout;
  const { Option } = Select;
  const router = useRouter();
  const [bidList, setBidList] = useState([]);
  const toOption = (bidList: number[]) => {
    const bidOptionList = [];
    for (let i = 0; i < bidList.length; i++) {
      bidOptionList.push(
        <Option key={bidList[i].toString()} value={bidList[i]}>
          {bidList[i].toString()}
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
  }, []);
  const onFinish = async (values: any) => {
    const data = {
      ...values,
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/post`,
        { ...data },
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
      alert(e);
    }
  };

  const onFinishFailed = (errorInfo: any) => {
    message.error(errorInfo);
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
            onFinishFailed={onFinishFailed}
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
              <TextArea rows={4} />
            </Form.Item>
            {/* <Form.Item> <Markdown/></Form.Item> */}
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
