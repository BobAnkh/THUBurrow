import React, { useEffect, useState } from 'react';
import Link from 'next/link';
import axios from 'axios';
import styles from '../pages/burrow/burrow.module.css';
import { Button, List, message, Space, Tag } from 'antd';
import {
  MessageOutlined,
  LikeOutlined,
  LikeTwoTone,
  StarOutlined,
  StarTwoTone,
} from '@ant-design/icons';

type IconProps = {
  icon?: any;
  text?: string;
};

type Props = {
  listData: any;
  setPage: any;
  totalNum: number;
};

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const IconText = ({ icon, text }: IconProps) => (
  <Space>
    {React.createElement(icon)}
    {text}
  </Space>
);
function showtag1(tag: string, index: number) {
  if (tag === '') return null;
  else return <Tag key={index}>{tag}</Tag>;
}
const showtag = (value: Array<string>) => {
  return (value || []).map(showtag1);
};
export default function PostList({ listData, setPage, totalNum }: Props) {
  const initialchange1 = new Array(20).fill(false);
  const initialchange2 = new Array(20).fill(false);
  const initialnum1 = new Array(20).fill(0);
  const initialnum2 = new Array(20).fill(0);
  const [changeLike, setChangeLike] = useState(initialchange1);
  const [changeCol, setChangeCol] = useState(initialchange2);
  const [likeNum, setLikeNum] = useState(initialnum1);
  const [colNum, setColNum] = useState(initialnum2);

  const clickCol = async (pid: number, activate: Boolean, index: number) => {
    let newChangeCol: boolean[] = changeCol;
    newChangeCol[index] = !changeCol[index];
    setChangeCol([...newChangeCol]);
    const newColNum = colNum;
    try {
      if (activate) {
        newColNum[index] = colNum[index] + 1;
        setColNum([...newColNum]);
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateCollection: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
        newColNum[index] = colNum[index] - 1;
        setColNum([...newColNum]);
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

  const clickLike = async (pid: number, activate: Boolean, index: number) => {
    let newChangeLike: boolean[] = changeLike;
    newChangeLike[index] = !changeLike[index];
    setChangeLike([...newChangeLike]);
    const newLikeNum = likeNum;
    try {
      if (activate) {
        newLikeNum[index] = likeNum[index] + 1;
        setLikeNum([...newLikeNum]);
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateLike: pid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
        newLikeNum[index] = likeNum[index] - 1;
        setLikeNum([...newLikeNum]);
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
        <List.Item
          key={item.post.title}
          actions={[
            <Button
              type='text'
              icon={
                (changeLike[index] && !item.like) ||
                (!changeLike[index] && item.like) ? (
                  <LikeTwoTone twoToneColor='#8A2BE2' />
                ) : (
                  <LikeOutlined />
                )
              }
              key='list-vertical-like-o'
              onClick={() => {
                clickLike(
                  item.post.post_id,
                  (!changeLike[index] && !item.like) ||
                    (changeLike[index] && item.like),
                  index
                );
              }}
            >
              {' '}
              {item.post.like_num + likeNum[index]}
            </Button>,
            <Button
              type='text'
              icon={
                (!changeCol[index] && item.collection) ||
                (changeCol[index] && !item.collection) ? (
                  <StarTwoTone twoToneColor='#FFD700' />
                ) : (
                  <StarOutlined />
                )
              }
              key='list-vertical-star-o'
              onClick={() => {
                clickCol(
                  item.post.post_id,
                  (!changeCol[index] && !item.collection) ||
                    (changeCol[index] && item.collection),
                  index
                );
              }}
            >
              {' '}
              {item.post.collection_num + colNum[index]}
            </Button>,
            <IconText
              icon={MessageOutlined}
              text={item.post.post_len}
              key='list-vertical-message'
            />,
          ]}
        >
          <List.Item.Meta
            title={
              <a href={`/post/${item.post.post_id}`} className={styles.Title}>
                # {item.post.post_id}&emsp;{item.post.title}
              </a>
            }
            description={`#${item.post.burrow_id} 洞主`}
          />
          {item.post.content}
          {showtag(item.post.tag)}
        </List.Item>
      )}
    />
  );
}

export function PostColList({ listData, setPage, totalNum }: Props) {
  const initialnum1 = new Array(20).fill(0);
  const initialnum2 = new Array(20).fill(0);
  const [likeNum, setLikeNum] = useState(initialnum1);
  const [colNum, setColNum] = useState(initialnum2);

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
        <List.Item
          key={item.post.title}
          actions={[
            <IconText
              text={item.post.like_num + likeNum[index]}
              icon={LikeOutlined}
              key='list-vertical-like-o'
            />,
            <IconText
              text={item.post.collection_num + colNum[index]}
              icon={StarOutlined}
              key='list-vertical-star-o'
            />,
            <IconText
              icon={MessageOutlined}
              text={item.post.post_len}
              key='list-vertical-message'
            />,
          ]}
        >
          <List.Item.Meta
            title={
              <Link href={`post/${item.post.post_id}`}>
                # {item.post.post_id}&emsp;{item.post.title}
              </Link>
            }
            description={`#${item.post.burrow_id} 洞主`}
          />
          {item.post.content}
          {showtag(item.post.tag)}
        </List.Item>
      )}
    />
  );
}
