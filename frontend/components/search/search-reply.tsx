import { List, Tag } from 'antd';
import styles from '../../styles/search.module.css';
import { Reply } from '../../models/search/data.d';
import moment from 'moment';

type Iprops = {
  replylist: any;
  loading: boolean;
  loadMoreDom: any;
};

export default function Searchreply({
  replylist,
  loading,
  loadMoreDom,
}: Iprops) {
  return (
    <List<Reply>
      loading={loading}
      loadMore={loadMoreDom}
      itemLayout='vertical'
      size='large'
      dataSource={replylist}
      footer={
        <div>
          <b>THU Burrow</b>
        </div>
      }
      renderItem={(item) => (
        <List.Item key={item.post_id}>
          <List.Item.Meta
            title={
              <a href={`/content/posts/${item.post_id}`}>帖#{item.post_id}</a>
            }
            description={`该帖子的回复`}
          />
          <List
            size='default'
            itemLayout='horizontal'
            dataSource={item.replies}
            renderItem={(item) => (
              <List.Item key={item.reply_id}>
                <List.Item.Meta
                  title={item.reply_id}
                  description={`#${item.burrow_id} 洞`}
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
        </List.Item>
      )}
    />
  );
}
