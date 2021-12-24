import { List } from 'antd';
import styles from '../../styles/search.module.css';
import { BurrowListItemDataType } from '../../models/search/data.d';
import moment from 'moment';
type Iprops = {
  burrowlist: any;
  loading: boolean;
  loadMoreDom: any;
};
export default function Searchburrow({
  burrowlist,
  loading,
  loadMoreDom,
}: Iprops) {
  return (
    <List<BurrowListItemDataType>
      loading={loading}
      key={1}
      loadMore={loadMoreDom}
      itemLayout='vertical'
      size='large'
      dataSource={burrowlist}
      footer={
        <div>
          <b>THU Burrow</b>
        </div>
      }
      renderItem={(item) => (
        <List.Item key={item.burrow_id}>
          <List.Item.Meta
            title={
              <a href={`/burrow/${item.burrow_id}`}>
                <div dangerouslySetInnerHTML={{ __html: item.title }}></div>
              </a>
            }
            description={
              item.status == false ? (
                <span>
                  <span>{`#${item.burrow_id} 洞主`}</span>
                  <strong>
                    <em> 已废弃</em>
                  </strong>
                </span>
              ) : (
                <span>{`#${item.burrow_id} 洞主`}</span>
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
  );
}
