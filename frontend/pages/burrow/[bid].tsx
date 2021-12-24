import React, { useState, useEffect } from 'react';
import type { NextPage } from 'next';
import { StarTwoTone, LikeTwoTone } from '@ant-design/icons';
import styles from './burrow.module.css';
import {
  Layout,
  List,
  Space,
  message,
  Form,
  Button,
  Input,
  Card,
  Tag,
  Popconfirm,
} from 'antd';
import {
  MessageOutlined,
  LikeOutlined,
  StarOutlined,
  PlusOutlined,
} from '@ant-design/icons';
import { useRouter } from 'next/router';
import 'antd/dist/antd.css';
import axios, { AxiosError } from 'axios';
import GlobalHeader from '../../components/header/header';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const { Header, Content, Footer } = Layout;
const { TextArea } = Input;

const IconText = (props: any) => (
  <Space>
    {React.createElement(props.icon)}
    {props.text}
  </Space>
);

function showtag(tag: string, index: number) {
  if (tag === '') return null;
  else return <Tag key={index}>{tag}</Tag>;
}
const show = (value: Array<string>, content: string) => {
  if (content === 'tag') return (value || []).map(showtag);
  else if (content === 'section') return value.map(showsection);
};

function showsection(tag: string, index: number) {
  return (
    <Tag key={index} color={'yellow'}>
      {tag}
    </Tag>
  );
}

const Burrow: NextPage = () => {
  const initialchange1 = new Array(10).fill(false);
  const initialchange2 = new Array(10).fill(false);
  const initialnum1 = new Array(10).fill(0);
  const initialnum2 = new Array(10).fill(0);

  const [listData, setListData] = useState([]);
  const [description, setDescription] = useState('Welcome!');
  const [burrowTitle, setBurrowTitle] = useState(0);
  const [page, setPage] = useState(1);
  const [isHost, setIsHost] = useState(false);

  const [editing, setEditing] = useState(false);
  const [descriptionTemp, setDescriptionTemp] = useState('');

  const [changeLike, setChangeLike] = useState(initialchange1);
  const [changeCol, setChangeCol] = useState(initialchange2);
  const [likeNum, setLikeNum] = useState(initialnum1);
  const [colNum, setColNum] = useState(initialnum2);

  const router = useRouter();
  const { bid } = router.query;
  const site = router.pathname.split('/')[1];

  const [attention, setattention] = useState(false);
  const [yourself, setyourself] = useState(false);

  const clickattention = async (bid: number, activate: Boolean) => {
    setattention(!attention);
    try {
      if (activate) {
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { ActivateFollow: bid },
          { headers: { 'Content-Type': 'application/json' } }
        );
      } else {
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/relation`,
          { DeactivateFollow: bid },
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

  useEffect(() => {
    try {
      const fetchListData = async () => {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/burrows/${bid}?page=${page - 1}`
        );
        const postlist = await res.data;
        setListData(postlist.posts);
        setDescription(postlist.description);
        setBurrowTitle(postlist.title);
        setIsHost(postlist.isHost);
      };
      fetchListData();
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status === 400) {
        message.info('请先登录!');
        router.push('/login');
      } else if (err.response?.status === 500) {
        message.info('服务器错误!');
        window.location.reload();
      }
    }
    axios
      .get(`${process.env.NEXT_PUBLIC_BASEURL}/users/valid-burrows`)
      .then(function (res) {
        res.data.indexOf(Number(bid)) == -1
          ? setyourself(false)
          : setyourself(true);
      });
    axios
      .get(`${process.env.NEXT_PUBLIC_BASEURL}/users/follow`)
      .then(function (res) {
        var j;
        for (j = 0; j < res.data.length; j++) {
          if (Number(bid) == res.data.burrow.burrow_id) {
            setattention(true);
            break;
          }
        }
      });
  }, [router, page]);

  const EditIntro = () => {
    setEditing(true);
  };

  const ConfirmEdit = async () => {
    setDescription(descriptionTemp);
    setEditing(false);
    const data = {
      title: { burrowTitle },
      description: { descriptionTemp },
    };
    try {
      const res = await axios.patch(
        `${process.env.NEXT_PUBLIC_BASEURL}/burrows/${bid}`,
        data
      );
      var json = await res.data;
      if (json.error) {
        message.error('修改信息失败');
        window.location.reload();
      } else {
        message.success('修改成功');
        window.location.reload();
      }
    } catch (e) {
      message.error('修改信息失败');
      alert(e);
    }
  };

  const CancelEdit = () => {
    setEditing(false);
  };

  const UpdateIntro = (event: any) => {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setDescriptionTemp(value);
    }
  };

  async function onConfirm() {
    try {
      const res = await axios.delete(
        `${process.env.NEXT_PUBLIC_BASEURL}/burrows/${bid}`
      );
      var json = await res.data;
      if (json.error) {
        message.error('操作失败');
        window.location.reload();
      } else {
        message.success('操作成功');
        window.location.reload();
      }
    } catch (e) {
      message.error('操作失败');
      alert(e);
    }
  }

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
    <Layout id='main'>
      <Header style={{ position: 'fixed', zIndex: 1, width: '100%' }}>
        <title className={styles.Title}>{`# ${bid} 地洞`}</title>
        <GlobalHeader />
      </Header>
      <Content
        className='site-Layout'
        style={{ padding: '0 50px', marginTop: 64 }}
      >
        <div
          className='site-layout-background'
          style={{ padding: 24, minHeight: 380 }}
        >
          <Card>
            <div>
              <h2>
                <table>
                  <tbody>
                    <tr>
                      <td style={{ width: '100%' }}>
                        <div className={styles.Title}>
                          # {bid}&emsp;{burrowTitle}
                        </div>
                        {yourself == false && (
                          <Button
                            icon={
                              !attention && (
                                <PlusOutlined twoToneColor='#FFD700' />
                              )
                            }
                            onClick={() => {
                              clickattention(Number(bid), attention);
                            }}
                            style={{ float: 'right', margin: '10px' }}
                          >
                            {attention == true ? '已关注' : '关注'}
                          </Button>
                        )}
                      </td>
                      <td>
                        <Popconfirm
                          placement='topRight'
                          title='确认废弃此洞?（此操作不可逆）'
                          onConfirm={onConfirm}
                          okText='Yes'
                          cancelText='No'
                        >
                          <Button
                            type='primary'
                            danger
                            shape='round'
                            style={{
                              display: isHost && !editing ? 'block' : 'none',
                            }}
                          >
                            废弃此洞
                          </Button>
                        </Popconfirm>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </h2>
              <div className={styles.Descript}>
                <h3 className={styles.BriefIntro}>简介:</h3>
                <Button
                  type='primary'
                  shape='round'
                  onClick={EditIntro}
                  style={{
                    float: 'right',
                    display: isHost && !editing ? 'block' : 'none',
                  }}
                >
                  编辑
                </Button>
                <div
                  style={{
                    paddingLeft: '35px',
                    display: editing ? 'none' : 'block',
                  }}
                >
                  {description}
                </div>
                <Form
                  style={{
                    paddingLeft: '35px',
                    display: editing ? 'block' : 'none',
                  }}
                >
                  <TextArea
                    autoSize={{ minRows: 2, maxRows: 6 }}
                    className={styles.EditText}
                    onChange={(event) => UpdateIntro(event)}
                    showCount
                    bordered={false}
                  />
                  <Button
                    className={styles.Cancel}
                    onClick={CancelEdit}
                    shape='round'
                  >
                    取消
                  </Button>
                  <Button
                    className={styles.Confirm}
                    type='primary'
                    shape='round'
                    onClick={ConfirmEdit}
                  >
                    确认
                  </Button>
                </Form>
              </div>
            </div>
            <List
              itemLayout='vertical'
              size='large'
              pagination={{
                onChange: (page) => {
                  setPage(page);
                },
                pageSize: 20,
              }}
              dataSource={listData}
              footer={
                <div>
                  <b>THUBurrow</b> footer part
                </div>
              }
              renderItem={(item: any, index: any) => (
                <List.Item
                  style={{
                    background: index % 2 === 0 ? '#f1f4f8' : '#FFFFFF',
                  }}
                  key={item.title}
                  actions={[
                    <Button
                      type='text'
                      icon={
                        (changeLike[index] && item.like) ||
                        (!changeLike[index] && !item.like) ? (
                          <LikeTwoTone twoToneColor='#8A2BE2' />
                        ) : (
                          <LikeOutlined />
                        )
                      }
                      key='list-vertical-like-o'
                      onClick={() => {
                        clickLike(
                          item.post_id,
                          (!changeLike[index] && item.like) ||
                            (changeLike[index] && !item.like),
                          index
                        );
                      }}
                      className={styles.ButtonLayout}
                    >
                      {' '}
                      {item.like_num + likeNum[index]}
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
                          item.post_id,
                          (changeCol[index] && item.collection) ||
                            (!changeCol[index] && !item.collection),
                          index
                        );
                      }}
                      className={styles.ButtonLayout}
                    >
                      {' '}
                      {item.collection_num + colNum[index]}
                    </Button>,
                    <IconText
                      icon={MessageOutlined}
                      text={item.post_len}
                      key='list-vertical-message'
                      className={styles.ButtonLayout}
                    />,
                  ]}
                >
                  <List.Item.Meta
                    title={
                      <a
                        href={`../post/${item.post_id}`}
                        className={styles.Title}
                      >
                        {item.title}&emsp;
                        {show(item.section, 'section')}
                      </a>
                    }
                  />
                  {show(item.tag, 'tag')}
                </List.Item>
              )}
            />
          </Card>
        </div>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default Burrow;
