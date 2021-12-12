import {
  LikeOutlined,
  DislikeOutlined,
  LoadingOutlined,
  MessageOutlined,
  StarOutlined,
} from '@ant-design/icons';
import {
  Layout,
  Button,
  Card,
  Col,
  Form,
  List,
  Row,
  Breadcrumb,
  Select,
  Tag,
  message,
} from 'antd';
import { FC, useEffect } from 'react';
import React from 'react';
import {
  PostListItemDataType,
  BurrowListItemDataType,
} from '../req/search/data.d';
import styles from '../styles/search.module.css';
import GlobalHeader from '../components/header/header';
import { Input } from 'antd';
import axios, { AxiosError } from 'axios';
import { useState } from 'react';
import { string } from 'prop-types';
import moment from 'moment';

const fakeDataUrl = `${process.env.NEXT_PUBLIC_BASEURL}/search`;
axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const { Option } = Select;
const { Search } = Input;
const { Content, Footer } = Layout;
const FormItem = Form.Item;
const IconText: React.FC<{
  type: string;
  text: React.ReactNode;
}> = ({ type, text }) => {
  switch (type) {
    case 'star-o':
      return (
        <span>
          <StarOutlined style={{ marginRight: 8 }} />
          {text}
        </span>
      );
    case 'like-o':
      return (
        <span>
          <LikeOutlined style={{ marginRight: 8 }} />
          {text}
        </span>
      );
    case 'dislike-o':
      return (
        <span>
          <DislikeOutlined style={{ marginRight: 8 }} />
          {text}
        </span>
      );
    case 'message':
      return (
        <span>
          <MessageOutlined style={{ marginRight: 8 }} />
          {text}
        </span>
      );
    default:
      return null;
  }
};
function showcontend1(value: any) {
  return <div>{value.title}</div>;
}
const showcontend = (value: any) => {
  return value.map(showcontend1);
};
function showtag1(tag: string) {
  return <Tag>{tag}</Tag>;
}
const showtag = (value: Array<string>) => {
  return value.map(showtag1);
};
const SearchPage: FC = () => {
  const [search_text, settext] = useState({
    keyword: ['', '', '', '', '', ''],
    id: 0,
    tag: ['', '', '', '', '', ''],
  });
  const [page, setpage] = useState(1);
  const [area, setarea] = useState('post');

  const [form] = Form.useForm();

  const [loading, setloading] = useState(false);
  const [loadingMore, setloadingMore] = useState(false);
  const [state, setstate] = useState('');

  const [listData, setdata] = useState<any[]>([]);
  const [found_number, setfound_number] = useState(0);

  async function sendkeyword(Keyword: any, page: number) {
    if (area === 'post') {
      const SearchPostKeyword = { keyword: Keyword };
      setloading(true);
      axios
        .post(`${process.env.NEXT_PUBLIC_BASEURL}/search?page=${page - 1}`, {
          SearchPostKeyword: SearchPostKeyword,
        })
        .then(function (res) {
          setdata(() => {
            setstate('post');
            setfound_number(res.data.posts.found);
            if (page == 1) return res.data.posts.posts;
            else return listData.concat(res.data.posts.posts);
          });
        })
        .catch(function (error) {
          const err = error as AxiosError;
          if (err.response?.status == 500) {
            message.error('服务器错误');
          }
        });
      setloading(false);
    } else {
      const SearchBurrowKeyword = { keyword: Keyword };
      setloading(true);
      axios
        .post(`${process.env.NEXT_PUBLIC_BASEURL}/search?page=${page - 1}`, {
          SearchBurrowKeyword: SearchBurrowKeyword,
        })
        .then(function (res) {
          setdata(() => {
            setstate('burrow');
            setfound_number(res.data.found);
            if (page == 1) return res.data.burrows;
            else return listData.concat(res.data.burrows);
          });
        })
        .catch(function (error) {
          const err = error as AxiosError;
          if (err.response?.status == 500) {
            message.error('服务器错误');
          }
        });
      setloading(false);
    }
  }

  async function sendid(id: number) {
    if (area == 'post') {
      const RetrievePost = { post_id: id };
      setloading(true);
      axios
        .post(`${process.env.NEXT_PUBLIC_BASEURL}/search`, {
          RetrievePost: RetrievePost,
        })
        .then(function (res) {
          var data1 = new Array();
          data1[0] = res.data.post_desc;
          setdata(() => {
            setstate('post');
            setfound_number(1);
            return data1;
          });
        })
        .catch(function (error) {
          const err = error as AxiosError;
          if (err.response?.status == 500) {
            message.error('服务器错误');
          }
          if (err.response?.status == 404) {
            message.error('找不到记录');
          }
        });
      setloading(false);
    } else {
      const RetrieveBurrow = { burrow_id: id };
      setloading(true);
      axios
        .post(`${process.env.NEXT_PUBLIC_BASEURL}/search`, {
          RetrieveBurrow: RetrieveBurrow,
        })
        .then(function (res) {
          var data1 = new Array();
          data1[0] = res.data;
          setdata(() => {
            setstate('burrow');
            setfound_number(1);
            return data1;
          });
        })
        .catch(function (error) {
          const err = error as AxiosError;
          if (err.response?.status == 500) {
            message.error('服务器错误');
          }
          if (err.response?.status == 404) {
            message.error('找不到记录');
          }
        });
      setloading(false);
    }
  }

  async function sendtag(tag: any, page: number) {
    const SearchPostTag = { tag: tag };
    setloading(true);
    axios
      .post(`${process.env.NEXT_PUBLIC_BASEURL}/search?page=${page - 1}`, {
        SearchPostTag: SearchPostTag,
      })
      .then(function (res) {
        setdata(() => {
          setstate('post');
          setfound_number(res.data.found);
          if (page == 1) return res.data.posts;
          else return listData.concat(res.data.posts);
        });
      })
      .catch(function (error) {
        const err = error as AxiosError;
        if (err.response?.status == 500) {
          message.error('服务器错误');
        }
      });
    setloading(false);
  }

  useEffect(() => {
    const params = {
      keyword: search_text.keyword,
      id: search_text.id,
      tag: search_text.tag,
      page: page,
      area: area,
    };
    if (params.tag.length > 0 && params.area === 'burrow') {
      params.keyword = params.tag;
      params.tag = [];
    }
    if (params.keyword.length > 0) {
      sendkeyword(params.keyword, params.page);
    }
    if (params.tag.length > 0) {
      sendtag(params.tag, params.page);
    }
    if (params.id != 0) {
      sendid(params.id);
    }
  }, [search_text, page]);

  const on_change_area = (data: string) => {
    setarea(data);
    setpage(1);
  };

  const loadMore = () => {
    setloadingMore(true);
    setpage(() => {
      return page + 1;
    });
    setloadingMore(false);
  };

  const loadMoreDom = listData.length > 0 && (
    <div style={{ textAlign: 'center', marginTop: 16 }}>
      <Button onClick={loadMore} style={{ paddingLeft: 48, paddingRight: 48 }}>
        {loadingMore ? (
          <span>
            <LoadingOutlined /> 加载中...
          </span>
        ) : (
          '加载更多'
        )}
      </Button>
    </div>
  );
  const handleFormSubmit = (value: string) => {
    if (value !== '' || value !== null) {
      if (value[0] == '#') {
        var reg = /#/;
        var reg1 = /^[0-9]*[0-9][0-9]*$/;
        if (reg1.test(value.replace(reg, ''))) {
          var value1 = Number(value.replace(reg, ''));
          settext({ keyword: [], id: value1, tag: [] });
          setpage(1);
        } else {
          if (value.replace(reg, '') !== '') {
            var str = value.replace(reg, '').split(/ ,/);
            if (str.length <= 6) settext({ keyword: [], id: 0, tag: str });
            else settext({ keyword: [], id: 0, tag: str.slice(0, 6) });
            setpage(1);
          }
        }
      } else {
        var str = value.split(/ ,/);
        if (str.length <= 6) settext({ keyword: str, id: 0, tag: [] });
        else settext({ keyword: str.slice(0, 6), id: 0, tag: [] });
        setpage(1);
      }
    }
  };

  const selectarea = (
    <Select
      style={{ width: '70px' }}
      placeholder={'范围'}
      onChange={on_change_area}
    >
      <Option value='burrow'>搜洞</Option>
      <Option value='post'>搜帖</Option>
    </Select>
  );
  function showtag1(tag: string) {
    return <Tag>{tag}</Tag>;
  }
  const showtag = (value: Array<string>) => {
    return value.map(showtag1);
  };
  return (
    <Layout className='layout'>
      <title>
        {search_text.tag.length == 0
          ? `${search_text.keyword[0]}……搜索`
          : `${search_text.tag[0]}……搜索`}
      </title>
      <GlobalHeader />
      <Content style={{ padding: '0 5%' }}>
        <Breadcrumb style={{ margin: '16px 0' }}>
          <Breadcrumb.Item>Home</Breadcrumb.Item>
          <Breadcrumb.Item>List</Breadcrumb.Item>
          <Breadcrumb.Item>App</Breadcrumb.Item>
        </Breadcrumb>

        <div className={styles.controlbar} style={{ textAlign: 'center' }}>
          <Search
            style={{ width: '300px' }}
            addonBefore={selectarea}
            placeholder={'关键词或#洞、帖号, tag'}
            allowClear
            onSearch={handleFormSubmit}
          />
        </div>

        <Card
          style={{ marginTop: 24 }}
          bordered={false}
          bodyStyle={{ padding: '8px 32px 32px 32px' }}
        >
          <p>
            找到<mark>{found_number}</mark>个结果
          </p>
          {state == 'post' ? (
            <List<PostListItemDataType>
              loading={loading}
              loadMore={loadMoreDom}
              itemLayout='vertical'
              size='large'
              dataSource={listData}
              footer={
                <div>
                  <b>THU Burrow</b>
                </div>
              }
              renderItem={(item) => (
                <List.Item>
                  <List.Item.Meta
                    title={
                      <a
                        href={`/contend/{${
                          item.post_id == undefined
                            ? search_text.id
                            : item.post_id
                        }}`}
                      >
                        帖{item.post_id}
                      </a>
                    }
                    description={
                      item.highlights !== undefined && (
                        <div>{item.highlights[0].snippet}</div>
                      )
                    }
                  />
                  {showtag(item.tag)}
                  <div className={styles.extra}>
                    {item.update_time !== undefined && (
                      <em>
                        updated at:{' '}
                        {moment(item.update_time).format('YYYY-MM-DD HH:mm')}
                      </em>
                    )}
                  </div>
                </List.Item>
              )}
            />
          ) : (
            <List<BurrowListItemDataType>
              loading={loading}
              loadMore={loadMoreDom}
              itemLayout='vertical'
              size='large'
              dataSource={listData}
              footer={
                <div>
                  <b>THU Burrow</b>
                </div>
              }
              renderItem={(item) => (
                <List.Item>
                  <List.Item.Meta
                    title={
                      <a
                        href={`/burrow/{${
                          item.burrow_id == undefined
                            ? search_text.id
                            : item.burrow_id
                        }}`}
                      >
                        洞
                        {item.burrow_id == undefined
                          ? search_text.id
                          : item.burrow_id}
                      </a>
                    }
                    description={
                      item.status == false ? (
                        <span>
                          {item.title}
                          <strong>
                            <em> 已废弃</em>
                          </strong>
                        </span>
                      ) : (
                        <span>{item.title}</span>
                      )
                    }
                  />
                  {item.highlights !== undefined && (
                    <div className={styles.description}>
                      <p>{item.highlights[0].snippet}</p>
                    </div>
                  )}
                  <div className={styles.listContent}>
                    <div className={styles.description}>{item.description}</div>
                    <div className={styles.extra}>
                      {item.update_time !== undefined && (
                        <em>
                          updated at:{' '}
                          {moment(item.update_time).format('YYYY-MM-DD HH:mm')}
                        </em>
                      )}
                    </div>
                  </div>
                </List.Item>
              )}
            />
          )}
        </Card>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default SearchPage;
