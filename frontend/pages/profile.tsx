import { ColumnsType, ColumnType } from 'antd/es/table';
import type { NextPage } from 'next';
import Link from 'next/link';
import { useRouter } from 'next/router';
import { FormInstance } from 'antd/lib/form';
import axios, { AxiosError } from 'axios';
import { PostColList } from '../components/post-list';
import { ExclamationCircleOutlined } from '@ant-design/icons';
import {
  Layout,
  Table,
  Badge,
  Divider,
  Modal,
  Form,
  Button,
  Input,
  message,
  Card,
} from 'antd';
import React, { useState, useEffect, useRef, useContext } from 'react';
import GlobalHeader from '../components/header/header';
import 'antd/dist/antd.css';
import '../styles/profile.module.css';
axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';
const { Header, Content, Footer } = Layout;

interface BurrowInfo {
  burrow_id: string;
  description: string;
  title: string;
  post_num: number;
}

interface EditableCellProps {
  title: React.ReactNode;
  editable: boolean;
  children: React.ReactNode;
  dataIndex: keyof MyBurrowInfo;
  record: MyBurrowInfo;
  handleSave: (record: MyBurrowInfo) => void;
  toggleEdit: (record: MyBurrowInfo) => void;
}

interface EditableRowProps {
  index: number;
}
interface MyBurrowInfo extends BurrowInfo {
  added?: boolean;
}

interface FollowedBurrowInfo extends BurrowInfo {
  is_update: boolean;
}

interface FollowedBurrowInfoRecv {
  is_update: boolean;
  burrow: BurrowInfo;
}
const burrowListMock: MyBurrowInfo[] = [
  {
    burrow_id: '',
    description: '',
    title: '',
    post_num: 0,
  },
];

const followedBurrowColumns: ColumnsType<FollowedBurrowInfo> = [
  {
    key: 'burrow_id',
    title: '洞号',
    dataIndex: 'burrow_id',
    width: '10%',
    render: (text, record, index) => {
      return record.is_update ? (
        <Badge dot>{text}</Badge>
      ) : (
        <Badge>{text}</Badge>
      );
    },
  },
  {
    key: 'title',
    title: '名称',
    width: '20%',
    dataIndex: 'title',
    render: (text, record, index) => {
      return <Link href={`/burrow/${record.burrow_id}`}>{text}</Link>;
    },
  },
  {
    key: 'description',
    title: '描述',
    width: '60%',
    dataIndex: 'description',
  },
  {
    key: 'post_num',
    title: '帖子数',
    dataIndex: 'post_num',
    width: '10%',
  },
];

type EditableTableProps = Parameters<typeof Table>[0];
type ColumnTypes = Exclude<EditableTableProps['columns'], undefined>;

const EditableContext = React.createContext<FormInstance<any> | null>(null);

const EditableRow: React.FC<EditableRowProps> = ({ index, ...props }) => {
  const [form] = Form.useForm(); //TODO: why the form is inside the editablerow?
  return (
    <Form form={form} component={false}>
      <EditableContext.Provider value={form}>
        <tr {...props} />
      </EditableContext.Provider>
    </Form>
  );
};

const EditableCell: React.FC<EditableCellProps> = ({
  title,
  editable,
  children,
  dataIndex,
  record,
  handleSave,
  ...restProps
}) => {
  const [editing, setEditing] = useState(record?.added ? true : false);
  const inputRef = useRef<Input>(null);
  const form = useContext(EditableContext)!;
  // if (record?.added){ 会触发infinite loop, why?
  //   setEditing(true)
  // }
  useEffect(() => {
    if (editing && !(record.added && title === '描述')) {
      inputRef.current!.focus();
    }
  }, [editing]);

  const toggleEdit = () => {
    setEditing(!editing);
    form.setFieldsValue({ [dataIndex]: record[dataIndex] });
  };

  const save = async () => {
    try {
      const values = await form.validateFields();
      // const values = record?.added? form.getFieldsValue(): await form.validateFields();

      toggleEdit();
      handleSave({ ...record, ...values });
    } catch (errInfo) {
      console.log('Save failed:', errInfo);
    }
  };

  let childNode = children;
  const placeholder =
    title === '洞号' ? '待分配' : title === '名称' ? '地洞名称' : '地洞描述';
  const tabIndex = title === '名称' ? 1 : 2;
  if (editable) {
    childNode = editing ? (
      <Form.Item
        style={{ margin: 0 }}
        name={dataIndex}
        rules={[
          {
            required: true,
            message: `请输入地洞${title}.`,
          },
        ]}
      >
        {record.added ? (
          <Input
            ref={inputRef}
            onPressEnter={save}
            onBlur={save}
            placeholder={placeholder}
            tabIndex={tabIndex}
          />
        ) : (
          <Input
            ref={inputRef}
            onPressEnter={save}
            onBlur={save}
            tabIndex={tabIndex}
          />
        )}
      </Form.Item>
    ) : (
      <div
        className='editable-cell-value-wrap'
        style={{ paddingRight: 24 }}
        onClick={toggleEdit}
      >
        {children}
      </div>
    );
  }

  return <td {...restProps}>{childNode}</td>;
};

const UserPage: NextPage = () => {
  const router = useRouter();
  const [followedList, setFollowedList] = useState([]);
  const [burrowList, setBurrowList] = useState<MyBurrowInfo[]>(burrowListMock);
  const [editingBurrow, setEditingBurrow] = useState<MyBurrowInfo>();
  const [page, setPage] = useState(1);
  const [totalPageNum, setTotalPageNum] = useState(1);
  const [postList, setPostList] = useState([]);
  const [editableList, setEditableList] = useState<{ [key: string]: boolean }>(
    burrowList.reduce((obj: { [key: string]: boolean }, x) => {
      obj[x.burrow_id] = false;
      return obj;
    }, {})
  );
  useEffect(() => {
    if (page > totalPageNum && postList.length == 20) {
      // 最后一页满则增大totalPageNum
      setTotalPageNum(page);
      console.log(`${totalPageNum},${postList.length}`);
    }
  }, [page]);
  // 获取关注的帖子
  useEffect(() => {
    const fetchPostList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/collection?page=${
            page - 1
          }`,
          // `http://127.0.0.1:4523/mock/435762/users/collection?page=${page - 1}`,
          {
            headers: { 'Content-Type': 'application/json' },
          }
        );
        const postlist = res.data.map(
          (obj: { post: any; is_update: boolean }) => obj.post
        );
        setPostList(postlist); //TODO: each child should have a unique key?
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchPostList();
  }, [page, router]);
  // 获取我的地洞
  useEffect(() => {
    const fetchBurrowList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/burrows`
        );
        const burrowlist = res.data;
        setBurrowList(burrowlist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchBurrowList();
  }, [router]);
  // 获取收藏的洞
  useEffect(() => {
    const fetchFollowedList = async () => {
      try {
        const res = await axios.get(
          `${process.env.NEXT_PUBLIC_BASEURL}/users/follow?page=0`
          // 'http://127.0.0.1:4523/mock2/435762/6973421'
        );
        const followedlist = res.data.map((obj: FollowedBurrowInfoRecv) => {
          return {
            is_update: obj.is_update,
            burrow_id: obj.burrow.burrow_id,
            title: obj.burrow.title,
            description: obj.burrow.description,
            post_num: obj.burrow.post_num,
          };
        });
        console.log('followedlist!');
        console.log(res.data);
        console.log(followedlist);
        setFollowedList(followedlist);
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    fetchFollowedList();
  }, [router]);

  const handleAdd = () => {
    const newData = [...burrowList!];
    const newEditableList = { ...editableList };
    if (Object.values(newEditableList).some((x) => x)) {
      Modal.info({
        title: '请先保存或取消当前编辑的洞!',
      });
    } else {
      newEditableList['待分配'] = true;
      const newRow: MyBurrowInfo = {
        burrow_id: '待分配',
        description: '',
        title: '',
        post_num: 0,
        added: true,
      };
      newData.push(newRow);
      setBurrowList(newData);
      setEditableList(newEditableList);
    }
  };

  const handleSaveCell = (row: MyBurrowInfo) => {
    //在button层面也handleSave
    const newData = [...burrowList!];
    const index = newData.findIndex((item) => row.burrow_id === item.burrow_id);
    const item = newData[index];
    newData.splice(index, 1, {
      ...item,
      ...row,
    });
    setBurrowList(newData);
  };

  const handleSave = (row: MyBurrowInfo) => {
    //在button层面也handleSave
    const postAdd = async () => {
      const data = {
        title: row.title,
        description: row.description,
      };
      try {
        const res = await axios.post(
          `${process.env.NEXT_PUBLIC_BASEURL}/burrows`,
          { ...data },
          { headers: { 'Content-Type': 'application/json' } }
        );
        const json = await res.data;
        console.log(json.burrow_id);
        newData[index].burrow_id = json.burrow_id;
        console.log(newData[index].burrow_id);
        delete newEditableList['待分配'];
        newEditableList[row.burrow_id] = false;
        setBurrowList(newData);
        setEditableList(newEditableList); //调用api保存树洞，获取新的洞号
        message.success('新建地洞成功!');
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
        } else if (err.response?.status == 403) {
          message.error('您被关进小黑屋啦，请遵守地洞规则哦，过一阵再建洞吧');
        } else if (err.response?.status == 429) {
          message.error('新建地洞达到上限，请明天再来吧qwq');
        } else if (err.response?.status == 500) {
          message.error('服务器出错啦orz');
        }
      }
    };
    const postUpdate = async () => {
      const data = {
        title: row.title,
        description: row.description,
      };
      try {
        const res = await axios.patch(
          `${process.env.NEXT_PUBLIC_BASEURL}/burrows/${row.burrow_id}`,
          { ...data },
          { headers: { 'Content-Type': 'application/json' } }
        );
        const json = await res.data;
        newEditableList[row.burrow_id] = false;
        setEditableList(newEditableList);
        message.success('更新地洞信息成功!');
      } catch (error) {
        const err = error as AxiosError;
        if (err.response?.status == 401) {
          // TODO: 处理各种错误类型
          message.error('请先登录！');
          router.push('/login');
        }
      }
    };
    const newEditableList = { ...editableList };
    newEditableList[row.burrow_id] = false;
    const newData = [...burrowList!];
    const index = burrowList!.findIndex(
      (item) => row.burrow_id === item.burrow_id
    );
    if (!(burrowList![index].title && burrowList![index].description)) {
      Modal.info({
        title: '请完善地洞信息!',
        onOk() {},
      });
    } else if (newData[index].added) {
      newData[index].added = false;
      postAdd();
    } else {
      postUpdate();
    }
  };
  const handleCancel = (row: MyBurrowInfo) => {
    const newEditableList = { ...editableList };
    newEditableList[row.burrow_id] = !newEditableList[row.burrow_id]; // 设置当前行为不可编辑
    Modal.confirm({
      title: 'Confirm',
      icon: <ExclamationCircleOutlined />,
      content: '取消编辑地洞信息吗吗',
      okText: '确认',
      cancelText: '取消',
      onOk: () => {
        const newData = [...burrowList];
        const index = burrowList.findIndex(
          (item) => row.burrow_id === item.burrow_id
        );
        if (row.added) {
          // 删除新增的行
          delete newEditableList[row.burrow_id];
          newData.splice(index, 1);
        } else {
          newData[index] = editingBurrow!;
        }
        setBurrowList(newData);
        setEditableList(newEditableList);
      },
    });
  };
  const handleEdit = (row: MyBurrowInfo) => {
    const newEditableList = { ...editableList };
    if (Object.values(newEditableList).some((x) => x)) {
      // 且存在(其他)可编辑的洞
      Modal.info({
        title: '请先保存或取消当前编辑的洞!',
      });
    } else {
      const index = burrowList.findIndex(
        (item) => row.burrow_id === item.burrow_id
      );
      setEditingBurrow(burrowList[index]); // TODO: I don't want to trigger re-render when change this, how should I do it?
      newEditableList[row.burrow_id] = !newEditableList[row.burrow_id]; // 设置当前行为可编辑
      setEditableList(newEditableList);
    }
  };

  const site = router.pathname.split('/')[1];
  type MyColumnsType<T> = (ColumnType<T> & { editable?: boolean })[];
  const myBurrowColumns: MyColumnsType<MyBurrowInfo> = [
    {
      key: 'burrow_id',
      title: '洞号',
      dataIndex: 'burrow_id',
      width: '10%',
    },
    {
      key: 'title',
      title: '名称',
      dataIndex: 'title',
      editable: true,
      width: '20%',
      render: (text, record, index) => {
        return <Link href={`/burrow/${record.burrow_id}`}>{text}</Link>;
      },
    },
    {
      key: 'description',
      title: '描述',
      dataIndex: 'description',
      editable: true,
      width: '40%',
    },
    {
      key: 'post_num',
      title: '帖子数',
      dataIndex: 'post_num',
      width: '10%',
    },
    {
      key: 'operation',
      title: '操作',
      dataIndex: 'operation',
      width: '20%',
      render: (_, row: MyBurrowInfo) =>
        editableList[row.burrow_id] ? (
          <span>
            <a onClick={() => handleSave(row)} style={{ marginRight: 8 }}>
              保存
            </a>
            <Divider type='vertical' />
            <a onClick={() => handleCancel(row)} style={{ marginRight: 8 }}>
              取消
            </a>
          </span>
        ) : (
          <span>
            <a onClick={() => handleEdit(row)}>编辑</a>
            {/* <Divider type="vertical" /> # only delete in burrow page
           <Popconfirm title="确定删除吗?" onConfirm={() => handleDelete(row)}>
             <a>删除</a>
           </Popconfirm> */}
          </span>
        ),
    },
  ];
  const components = {
    body: {
      row: EditableRow,
      cell: EditableCell,
    },
  };
  const columns = myBurrowColumns.map((col) => {
    if (!col.editable) {
      return col;
    }
    return {
      ...col,
      onCell: (record: BurrowInfo) => ({
        record,
        editable: col.editable && editableList[record.burrow_id],
        dataIndex: col.dataIndex,
        title: col.title,
        handleSave: handleSaveCell,
      }),
    };
  });
  return (
    <Layout className='layout'>
      <Header>
        <title> 个人主页 </title>
        <GlobalHeader />
      </Header>
      <Content>
        <Card title='我的地洞'>
          <Table
            components={components}
            columns={columns as ColumnTypes}
            dataSource={burrowList}
            rowKey='id'
            footer={() => (
              <Button
                type='dashed'
                style={{ width: '100%' }}
                onClick={handleAdd}
              >
                + 新增地洞
              </Button>
            )}
          />
        </Card>
        <Card title='关注的洞'>
          <Table<FollowedBurrowInfo>
            columns={followedBurrowColumns}
            dataSource={followedList}
            rowKey='id'
          />
        </Card>
        <Card title='收藏的帖子'>
          <PostColList
            listData={postList}
            setPage={setPage}
            totalNum={
              totalPageNum >= page
                ? (totalPageNum + 1) * 20
                : postList.length === 20
                ? (page + 1) * 20
                : page * 20
            }
          />
        </Card>
      </Content>
    </Layout>
  );
};

export default UserPage;
