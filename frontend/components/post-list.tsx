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
  setPage: any;
};

const IconText = ({ icon, text }: IconProps) => (
  <Space>
    {React.createElement(icon)}
    {text}
  </Space>
);

export default function PostList({ listData, setPage }: Props) {
  return (
    <List
      itemLayout='vertical'
      size='large'
      pagination={{
        onChange: (page) => {
          setPage(page);
        },
        pageSize: 3,
      }}
      dataSource={listData}
      footer={
        <div>
          <b>ant design</b> footer part
        </div>
      }
      renderItem={(item: any) => (
        <List.Item
          key={item.title}
          actions={[
            <IconText
              icon={StarOutlined}
              text={item.star}
              key='list-vertical-star-o'
            />,
            <IconText
              icon={LikeOutlined}
              text={item.like}
              key='list-vertical-like-o'
            />,
            <IconText
              icon={DislikeOutlined}
              text={item.dislike}
              key='list-vertical-dislike-o'
            />,
            <IconText
              icon={MessageOutlined}
              text={item.comment}
              key='list-vertical-message'
            />,
          ]}
        >
          <List.Item.Meta
            title={<Link href={`./post/${item.post_id}`}>{item.title}</Link>}
            description={item.author}
          />
          {item.content}
        </List.Item>
      )}
    />
  );
}
