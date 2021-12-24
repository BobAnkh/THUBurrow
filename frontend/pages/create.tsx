import React, { useState, useEffect } from 'react';
import { Button, Card, Form, Input, Layout, message, Select } from 'antd';
import TextArea from 'antd/lib/input/TextArea';
import axios, { AxiosError } from 'axios';
import { NextPage } from 'next';
import GlobalHeader from '../components/header/header';
import { useRouter } from 'next/router';
import { Markdown } from '../components';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const Create: NextPage = () => {
  const { Header, Content, Footer } = Layout;
  const { Option } = Select;
  const router = useRouter();
  const LIMIT_SIZE = 500 * 1000; //图片最大500kb
  const MAX_WIDTH = 2000;
  const MAX_HEIGHT = 2000;
  let compressCount = 0;
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
    if (newURL != '') {
      const newContent =
        content +
        `<img src='${newURL.slice(0, -5)}](${
          process.env.NEXT_PUBLIC_BASEURL
        }/storage/images/${newURL}' style='width : 60%'/>`;
      setContent(newContent);
    }
  }, [newURL]);

  const handleOnChange = (text: string) => {
    setContent(text);
  };

  const onFinish = async (values: any) => {
    const data = {
      ...values,
    };
    if (data.tag === undefined) data.tag = [];
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/posts`,
        { ...data },
        { headers: { 'Content-Type': 'application/json' } }
      );
      message.success('发帖成功');
      router.push('/home');
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

  const selectSection = (rule: any, value: any, callback: any) => {
    if (value.length > 3) {
      callback('wrong');
    } else {
      callback();
    }
  };

  const chooseTag = (rule: any, value: any, callback: any) => {
    if (value === undefined) callback();
    if (value.length > 10) {
      callback('wrong');
    } else {
      callback();
    }
  };

  const compress = (base64: any, quality: number, mimeType: string) => {
    console.log(base64);
    let canvas = document.createElement('canvas');
    let img = document.createElement('img');
    img.crossOrigin = 'anonymous';
    return new Promise<string>((resolve, reject) => {
      img.src = base64;
      img.onload = () => {
        let targetWidth, targetHeight;
        targetWidth = img.width;
        targetHeight = img.height;
        if (img.width > MAX_WIDTH) {
          targetWidth = MAX_WIDTH;
          targetHeight = (img.height * MAX_WIDTH) / img.width;
        }
        if (img.height > MAX_HEIGHT) {
          targetWidth = (img.width * MAX_HEIGHT) / img.height;
          targetHeight = MAX_HEIGHT;
        }
        canvas.width = targetWidth;
        canvas.height = targetHeight;
        let ctx = canvas.getContext('2d');
        ctx!.clearRect(0, 0, targetWidth, targetHeight); // 清除画布
        ctx!.drawImage(img, 0, 0, canvas.width, canvas.height);
        let imageData = canvas.toDataURL(mimeType, quality / 100); // 设置图片质量
        if (imageData.length - (imageData.length / 8) * 2 > LIMIT_SIZE) {
          compressCount += 1;
          compress(imageData, 0.9 - compressCount * 0.1, 'image/jpeg');
        } else {
          compressCount = 0;
          resolve(imageData);
        }
      };
    });
  };

  // 转化为Uint8Array
  function dataUrlToBlob(base64: string) {
    let bytes = window.atob(base64.split(',')[1]);
    let ab = new ArrayBuffer(bytes.length);
    let ia = new Uint8Array(ab);
    for (let i = 0; i < bytes.length; i++) {
      ia[i] = bytes.charCodeAt(i);
    }
    return ia;
  }

  const upLoadToServer = async (bytes: Uint8Array, type: string) => {
    try {
      console.log('size', bytes.length);
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/storage/images`,
        bytes,
        { headers: { 'Content-Type': type } }
      );
      setNewURL(res.data);
    } catch (e) {
      message.error('上传图片失败！');
    }
  };

  const uploadImage = (event: any) => {
    const reader = new FileReader();
    console.log('step0 done!');
    reader.onload = async function (event) {
      let compressedDataURL = await compress(
        event.target?.result,
        90,
        'image/jpeg'
      );
      let compressedImageUint8 = dataUrlToBlob(compressedDataURL);
      upLoadToServer(compressedImageUint8, 'image/jpeg');
    };
    reader.readAsDataURL(event.target.files[0]);
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
              <input type='file' accept='image/*' onChange={uploadImage} />
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
                rules={[
                  {
                    required: true,
                    message: '请选择分区',
                  },
                  { message: '至多选择3个分区', validator: selectSection },
                ]}
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
            <Form.Item
              label='Tag'
              name='tag'
              rules={[{ validator: chooseTag, message: '至多选择10个tag' }]}
            >
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
