import { List, Tag } from 'antd';
import styles from '../../styles/search.module.css';
import { Postreply } from '../../models/search/data.d';
import moment from 'moment';
import {
  LikeOutlined,
  DislikeOutlined,
  MessageOutlined,
  StarOutlined,
} from '@ant-design/icons';

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

type Iprops = {
  post_id: number;
  title: string;
  postreply: any;
};

export default function Searchpostid({ post_id, title, postreply }: Iprops) {
  return (
    <List<Postreply>
      key={post_id}
      itemLayout='vertical'
      size='large'
      dataSource={postreply}
      header={
        <div>
          <a href={`/post/{${post_id}}`}>帖#{post_id} </a>
          <b>{title}</b>
        </div>
      }
      footer={
        <div>
          <b>THU Burrow</b>
        </div>
      }
      renderItem={(item) => (
        <List.Item key={item.reply_id}>
          <List.Item.Meta
            title={<a href={`/burrow/{${item.burrow_id}}`}>{item.reply_id}</a>}
          />
          {item.content}
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
  );
}
