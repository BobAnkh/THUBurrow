import { List, Tag } from 'antd';
import styles from '../../styles/search.module.css';
import { PostListItemDataType } from '../../models/search/data.d';
import moment from 'moment';
import Link from 'next/link';

function showtag1(tag: string, index: number) {
  return <Tag key={index}>{tag}</Tag>;
}
const showtag = (value: Array<string>) => {
  return value.map(showtag1);
};
function showsection1(tag: string) {
  return <div> {tag} </div>;
}
const showsection = (value: Array<string>) => {
  return value.map(showsection1);
};

type Iprops = {
  tag: string;
  postlist: any;
  loading: boolean;
  loadMoreDom: any;
};

export default function Searchpost({
  postlist,
  loading,
  loadMoreDom,
  tag,
}: Iprops) {
  return (
    <List<PostListItemDataType>
      loading={loading}
      loadMore={loadMoreDom}
      itemLayout='vertical'
      size='large'
      header={tag != '' && <div>{tag}的帖子</div>}
      dataSource={postlist}
      footer={
        <div>
          <b>THU Burrow</b>
        </div>
      }
      renderItem={(item) => (
        <List.Item key={item.post_id}>
          <List.Item.Meta
            title={<Link href={`post/${item.post_id}`}>{item.title}</Link>}
            description={`#${item.burrow_id} 洞主`}
          />
          {item.tag != null && showtag(item.tag)}
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
