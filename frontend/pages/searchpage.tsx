import { LoadingOutlined } from '@ant-design/icons';
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
import styles from '../styles/search.module.css';
import GlobalHeader from '../components/header/header';
import { Input } from 'antd';
import axios, { AxiosError } from 'axios';
import { useState } from 'react';
import Searchburrow from '../components/search/search-burrow';
import Searchburrowid from '../components/search/search-burrowid';
import Searchpost from '../components/search/search-post';
import Searchreply from '../components/search/search-reply';
import Searchpostid from '../components/search/search-postid';
import { string } from 'prop-types';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const { Option } = Select;
const { Search } = Input;
const { Header, Content, Footer } = Layout;
const FormItem = Form.Item;

const SearchPage: FC = () => {
  const [search_text, settext] = useState({
    keyword: new Array(),
    id: 0,
    tag: new Array(),
  });
  const [page, setpage] = useState(1);
  const [area, setarea] = useState('post');

  const [form] = Form.useForm();

  const [loading, setloading] = useState(false);
  const [loadingMore, setloadingMore] = useState(false);
  const [state, setstate] = useState('');

  const [title, setitle] = useState('');
  const [description, setdescription] = useState('');

  const [replydata, setreply] = useState<any[]>([]);
  const [listData, setdata] = useState(new Array());
  const [found_number, setfound_number] = useState(0);
  const [found_number1, setfound_number1] = useState(0);
  const [showreply, setshowreply] = useState(false);

  async function sendkeyword(Keyword: any, page: number) {
    if (area === 'post') {
      const SearchPostKeyword = { keyword: Keyword };
      setloading(true);
      axios
        .post(`${process.env.NEXT_PUBLIC_BASEURL}/search?page=${page - 1}`, {
          SearchPostKeyword: SearchPostKeyword,
        })
        .then(function (res) {
          setstate('post');
          setreply(() => {
            setfound_number1(res.data.replies.found);
            if (page == 1) return res.data.replies.replies;
            else return replydata.concat(res.data.replies.replies);
          });
          setdata(() => {
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
            setstate(() => {
              return 'burrow';
            });
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
          setitle(res.data.post_desc.title);
          setreply(() => {
            setstate('post');
            setfound_number(1);
            return res.data.reply_page;
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
            setstate(() => {
              return 'burrow';
            });
            setdescription(res.data.description);
            setitle(res.data.title);
            setfound_number(1);
            return res.data.posts[0];
          });
        })
        .catch(function (error) {
          const err = error as AxiosError;
          if (err.response?.status == 500) {
            message.error('服务器错误');
          }
          if (err.response?.status == 404) {
            setfound_number(0);
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
    if (params.tag[0] != null && params.area === 'burrow') {
      params.keyword = params.tag;
      params.tag = [];
      sendkeyword(params.keyword, params.page);
    } else if (params.keyword[0] != null) {
      sendkeyword(params.keyword, params.page);
    } else if (params.tag[0] != null) {
      sendtag(params.tag, params.page);
    } else if (params.id != 0) {
      sendid(params.id);
    }
  }, [search_text, page]);

  const on_change_area = (data: string) => {
    setarea(() => {
      return data;
    });
  };

  const on_change_show = (data: string) => {
    if (data == 'replies') {
      setshowreply(true);
    } else {
      setshowreply(false);
    }
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
    if (value.length != 0) {
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
      defaultValue='post'
      onChange={on_change_area}
    >
      <Option value='burrow'>搜洞</Option>
      <Option value='post'>搜帖</Option>
    </Select>
  );
  function Switch() {
    if (state == 'burrow' && search_text.id == 0) {
      return (
        <Searchburrow
          burrowlist={listData}
          loadMoreDom={loadMoreDom}
          loading={loading}
        />
      );
    } else if (state == 'burrow' && search_text.id != 0) {
      return (
        <Searchburrowid
          burrow_id={search_text.id}
          title={title}
          description={description}
          burrowpost={listData}
        />
      );
    } else if (
      state == 'post' &&
      search_text.id == 0 &&
      showreply == true &&
      search_text.tag[0] == null
    ) {
      return (
        <Searchreply
          replylist={replydata}
          loadMoreDom={loadMoreDom}
          loading={loading}
        />
      );
    } else if (
      state == 'post' &&
      search_text.id == 0 &&
      showreply == false &&
      search_text.tag[0] == null
    ) {
      return (
        <Searchpost
          tag=''
          postlist={listData}
          loading={loading}
          loadMoreDom={loadMoreDom}
        />
      );
    } else if (state == 'post' && search_text.tag[0] != null) {
      return (
        <Searchpost
          tag={search_text.tag[0]}
          postlist={listData}
          loading={loading}
          loadMoreDom={loadMoreDom}
        />
      );
    } else if (state == 'post' && search_text.id != 0) {
      return (
        <Searchpostid
          postreply={replydata}
          title={title}
          post_id={search_text.id}
        />
      );
    }
  }

  return (
    <Layout className='layout'>
      <Header>
        <title>
          {search_text.tag.length == 0
            ? `${search_text.keyword[0]}……搜索`
            : `${search_text.tag[0]}……搜索`}
        </title>
        <GlobalHeader />
      </Header>
      <Content style={{ padding: '0 5%' }}>
        <div
          className={styles.controlbar}
          style={{ textAlign: 'center', margin: '16px 0', padding: '0 5%' }}
        >
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
          {search_text.keyword.length > 0 && state === 'post' && (
            <Select
              style={{ width: '170px' }}
              onChange={on_change_show}
              defaultValue='posts'
            >
              <Option value='posts'>查看帖子</Option>
              <Option value='replies'>查看回复</Option>
            </Select>
          )}
          <p>
            找到<mark>{showreply == false ? found_number : found_number1}</mark>
            个结果
          </p>
          <div> {Switch()}</div>
        </Card>
      </Content>
      <Footer style={{ textAlign: 'center' }}>THUBurrow © 2021</Footer>
    </Layout>
  );
};

export default SearchPage;
