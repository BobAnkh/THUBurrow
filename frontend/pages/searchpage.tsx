import {
  LikeOutlined,
  DislikeOutlined,
  LoadingOutlined,
  MessageOutlined,
  StarOutlined,
} from '@ant-design/icons';
import { Button, Card, Col, Form, List, Row, Select, Tag } from 'antd';
import { FC, useEffect } from 'react';
import React from 'react';
import {
  Params,
  PostListItemDataType,
  BurrowListItemDataType,
} from '../req/search/data.d';
import PBListContent from '../components/pbListContent/post-or-burrow-list';
import StandardFormRow from '../components/standardFormRow/standard-form-row';
import styles from '../styles/search.module.css';
import GlobalHeader from '../components/header/header';
import { Input } from 'antd';
import axios from 'axios';
import { useState } from 'react';

const fakeDataUrl = `${process.env.NEXT_PUBLIC_BASEURL}/search`;

const { Option } = Select;
const { Search } = Input;
const FormItem = Form.Item;

const SearchPage: FC = () => {
  const [search_text, settext] = useState({ keyword: '', id: 0, tag: '' });
  const [page, setpage] = useState(1);
  const [order, setorder] = useState('time');
  const [area, setarea] = useState('post');

  const [form] = Form.useForm();

  const [loading, setloading] = useState(false);
  const [loadingMore, setloadingMore] = useState(false);
  const [state, setstate] = useState('');

  const [listData, setdata] = useState([]);
  const [found_number, setfound_number] = useState(0);

  async function getData(params: Params) {
    if (params.keyword != '' || params.id != 0 || params.tag != '') {
      setloading(true);
      axios.post(fakeDataUrl, { body: params }).then(function (res) {
        setdata(() => {
          if (res.data.hits[0].document.burrow_id != null) {
            setstate('burrow');
          } else setstate('post');
          setfound_number(res.data.found);
          return res.data.hits;
        });
      });
      setloading(false);
    }
  }

  useEffect(() => {
    const params = {
      keyword: search_text.keyword,
      id: search_text.id,
      tag: search_text.tag,
      order: order,
      area: area,
      page: 1,
    };
    if (params.tag !== '' && params.area === 'burrow') {
      params.keyword = params.tag;
      params.tag = '';
    }
    getData(params);
  }, [search_text, area, order]);

  const on_change_area = (data: string) => {
    setarea(data);
    setpage(1);
  };
  const on_change_order = (data: string) => {
    setorder(data);
    setpage(1);
  };

  const loadMore = () => {
    setloadingMore(true);
    const params = {
      keyword: search_text.keyword,
      id: search_text.id,
      tag: search_text.tag,
      order: order,
      area: area,
      page: page + 1,
    };
    setpage(() => {
      return page + 1;
    });
    axios.post(fakeDataUrl, { body: params }).then(function (res) {
      setdata(() => {
        return listData.concat(res.data.hits);
      });
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
          settext({ keyword: '', id: value1, tag: '' });
          setpage(1);
        } else {
          if (value.replace(reg, '') !== '') {
            settext({ keyword: '', id: 0, tag: value.replace(reg, '') });
            setpage(1);
          }
        }
      } else {
        settext({ keyword: value, id: 0, tag: '' });
        setpage(1);
      }
    }
  };

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

  const formItemLayout = {
    wrapperCol: {
      xs: { span: 24 },
      sm: { span: 24 },
      md: { span: 12 },
    },
  };
  const selectarea = (
    <Select placeholder={'搜索范围'} onChange={on_change_area}>
      <Option value='搜洞'>搜洞</Option>
      <Option value='搜帖'>搜帖</Option>
    </Select>
  );

  return (
    <>
      <GlobalHeader />
      <div className={styles.controlbar}>
        <Search
          style={{ width: '400px' }}
          addonBefore={selectarea}
          placeholder={'搜索关键词 或 #洞号，帖号'}
          allowClear
          onSearch={handleFormSubmit}
        />
      </div>

      <Card bordered={false}>
        <Form layout='inline' form={form}>
          <p>找到{found_number}个结果</p>
          <StandardFormRow grid last>
            <Row gutter={16}>
              <Col xl={8} lg={10} md={12} sm={24} xs={24}>
                <FormItem {...formItemLayout} label='排序方式' name='order'>
                  <Select
                    placeholder='按时间'
                    style={{ maxWidth: 200, width: '100%' }}
                    onChange={on_change_order}
                  >
                    <Option value='time'>按时间</Option>
                    <Option value='heat'>按热度</Option>
                  </Select>
                </FormItem>
              </Col>
            </Row>
          </StandardFormRow>
        </Form>
      </Card>
      <Card
        style={{ marginTop: 24 }}
        bordered={false}
        bodyStyle={{ padding: '8px 32px 32px 32px' }}
      >
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
              <List.Item
                actions={[
                  <IconText
                    key='star'
                    type='star-o'
                    text={item.document.star}
                  />,
                  <IconText
                    key='like'
                    type='like-o'
                    text={item.document.like}
                  />,
                  <IconText
                    key='dislike'
                    type='dislike-o'
                    text={item.document.dislike}
                  />,
                  <IconText
                    key='message'
                    type='message'
                    text={item.document.message}
                  />,
                ]}
              >
                <List.Item.Meta
                  title={
                    <a href={item.document.url}>帖{item.document.post_id}</a>
                  }
                  description={
                    <span>
                      <Tag>{item.document.description.tag1}</Tag>
                      <Tag>{item.document.description.tag2}</Tag>
                      <Tag>{item.document.description.tag3}</Tag>
                    </span>
                  }
                />
                {item.highlights !== undefined && (
                  <div>{item.highlights[0].snippet}</div>
                )}
                <PBListContent data={item.document} />
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
              <List.Item
                actions={[
                  <IconText
                    key='star'
                    type='star-o'
                    text={item.document.star}
                  />,
                  <IconText
                    key='message'
                    type='message'
                    text={item.document.post_number}
                  />,
                ]}
              >
                <List.Item.Meta
                  title={
                    <a href={item.document.url}>洞{item.document.burrow_id}</a>
                  }
                  description={
                    item.document.status == false ? (
                      <span>
                        {item.document.title}
                        <strong>
                          <em> 已废弃</em>
                        </strong>
                      </span>
                    ) : (
                      <span>{item.document.title}</span>
                    )
                  }
                />
                {item.highlights !== undefined && (
                  <div className={styles.description}>
                    <p>{item.highlights[0].snippet}</p>
                  </div>
                )}
                <PBListContent data={item.document} />
              </List.Item>
            )}
          />
        )}
      </Card>
    </>
  );
};

export default SearchPage;
