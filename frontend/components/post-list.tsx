import React from 'react';
import Link from 'next/link';
import { List, Space } from 'antd';
import {
  MessageOutlined,
  LikeOutlined,
  StarOutlined,
  DislikeOutlined,
} from '@ant-design/icons';

type IconProps = {
  icon?: any;
  text?: string;
};

type Props = {
  listData: any;
  postNum: number;
  setPage: any;
};

const IconText = ({ icon, text }: IconProps) => (
  <Space>
    {React.createElement(icon)}
    {text}
  </Space>
);

export default function PostList({ listData, postNum, setPage }: Props) {
  return (
    <List
      itemLayout='vertical'
      size='large'
      pagination={{
        onChange: (page) => {
          setPage(page);
        },
        pageSize: 10,
        showQuickJumper: true,
        showSizeChanger: false,
        total: postNum,
      }}
      dataSource={listData}
      footer={
        <div>
          <b>THU Burrow</b> footer part
        </div>
      }
      renderItem={(item: any) => (
        <List.Item
          key={item.title}
          actions={[
            <IconText
              icon={StarOutlined}
              text={item.collection_num}
              key='list-vertical-star-o'
            />,
            <IconText
              icon={LikeOutlined}
              text={item.like_num}
              key='list-vertical-like-o'
            />,
            <IconText
              icon={MessageOutlined}
              text={item.post_len}
              key='list-vertical-message'
            />,
          ]}
        >
          <List.Item.Meta
            title={<Link href={`post/${item.post_id}`}>{item.title}</Link>}
            description={item.author}
          />
          {item.content}
        </List.Item>
      )}
    />
  );
}
