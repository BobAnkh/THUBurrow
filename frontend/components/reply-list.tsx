import React from 'react';
import { List } from 'antd';

type Props = {
  listData: any;
  setPage: any;
};

export default function ReplyList({ listData, setPage }: Props) {
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
        total: 2000,
      }}
      dataSource={listData}
      footer={
        <div>
          <b>THU Burrow</b> footer part
        </div>
      }
      renderItem={(item: any) => (
        <List.Item key={item.reply_id}>
          <List.Item.Meta
            title={
              <a
                href={`/burrow/${item.burrow_id}`}
              >{`#${item.burrow_id} 洞主`}</a>
            }
            description={`#${item.reply_id}`}
          />
          {item.content}
        </List.Item>
      )}
    />
  );
}
