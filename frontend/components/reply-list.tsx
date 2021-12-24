import React, { useState } from 'react';
import { Button, Form, List, message, Space } from 'antd';
import axios, { AxiosError } from 'axios';
import Markdown, { MarkdownViewer } from './markdown/markdown';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

type Props = {
  listData: any;
  setPage: any;
  userBid: number[];
  totalNum: number;
};

export default function ReplyList({
  listData,
  userBid,
  setPage,
  totalNum,
}: Props) {
  const initialEdit = new Array(20).fill(false);
  const [edit, setEdit] = useState(initialEdit);
  const [content, setContent] = useState('');
  const [mode, setMode] = useState<'view' | 'edit'>('edit');

  const onEdit = (initialValue: string, index: number) => {
    let newEdit: boolean[] = edit;
    newEdit[index] = true;
    setEdit([...newEdit]);
    setContent(initialValue);
  };

  const handleOnChange = (text: string) => {
    setContent(text);
  };

  const onSave = async (values: any, index: number) => {
    try {
      const res = await axios.patch(
        `${process.env.NEXT_PUBLIC_BASEURL}/content/replies`,
        {
          post_id: listData[0].post_id,
          reply_id: listData[0].reply_id,
          ...values,
        }
      );
      let newEdit: boolean[] = edit;
      newEdit[index] = false;
      setEdit([...newEdit]);
      message.success('修改成功');
      window.location.reload();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status == 400) {
        message.error('请求有误！');
      } else if (err.response?.status == 403) {
        message.error('禁止修改！');
      } else {
        message.error('未知错误！');
      }
    }
  };

  const onCancel = (index: number) => {
    let newEdit: boolean[] = edit;
    newEdit[index] = false;
    setEdit([...newEdit]);
  };

  return (
    <List
      itemLayout='vertical'
      size='large'
      pagination={{
        onChange: (page) => {
          setPage(page);
        },
        pageSize: 20,
        showQuickJumper: true,
        showSizeChanger: false,
        total: totalNum,
      }}
      dataSource={listData}
      renderItem={(item: any, index: number) => (
        <List.Item key={item.reply_id}>
          <List.Item.Meta
            title={
              <a
                href={`/burrow/${item.burrow_id}`}
              >{`#${item.burrow_id} 洞主`}</a>
            }
            description={`#${item.reply_id}`}
          />
          {userBid.indexOf(item.burrow_id) === -1 ? (
            MarkdownViewer(item.content)
          ) : edit[index] === false ? (
            <>
              {MarkdownViewer(item.content)}
              <Button
                type='link'
                htmlType='button'
                onClick={() => {
                  onEdit(item.content, index);
                }}
                style={{ float: 'right' }}
              >
                编辑
              </Button>
              <br />
            </>
          ) : (
            <Form
              onFinish={(values) => {
                onSave(values, index);
              }}
            >
              <Form.Item
                name='content'
                rules={[{ required: true, message: '回复不能为空！' }]}
              >
                <Markdown
                  content={content}
                  mode={mode}
                  editorStyle={{ height: '500px' }}
                  onChange={handleOnChange}
                />
              </Form.Item>
              <Form.Item>
                <Button
                  type='link'
                  htmlType='button'
                  onClick={() => {
                    onCancel(index);
                  }}
                  style={{ float: 'right' }}
                >
                  取消
                </Button>
                <Button
                  type='link'
                  style={{ float: 'right' }}
                  htmlType='submit'
                >
                  确认修改
                </Button>
              </Form.Item>
            </Form>
          )}
        </List.Item>
      )}
    />
  );
}
