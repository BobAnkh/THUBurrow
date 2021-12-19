import { List, Tag } from 'antd';
import { BurrowDataType } from '../../models/search/data.d';
import {
  LikeOutlined,
  DislikeOutlined,
  MessageOutlined,
  StarOutlined,
} from '@ant-design/icons';
type Iprops = {
  burrow_id: number;
  title: string;
  description: string;
  burrowpost: any;
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

export default function Searchburrowid({
  title,
  burrow_id,
  description,
  burrowpost,
}: Iprops) {
  return (
    <List<BurrowDataType>
      key={burrow_id}
      itemLayout='vertical'
      size='large'
      dataSource={burrowpost}
      header={
        <div>
          <a href={`/burrow/{${burrow_id}}`}> 洞#{burrow_id} </a>
          <b> {title} </b>
          <p> {description} </p>
        </div>
      }
      footer={
        <div>
          <b>THU Burrow</b>
        </div>
      }
      renderItem={(item) => (
        <List.Item
          key={item.post_id}
          actions={[
            <IconText key='star' type='star-o' text={item.collection_num} />,
            <IconText key='like' type='like-o' text={item.like_num} />,
            <IconText key='message' type='message' text={item.post_len} />,
          ]}
        >
          <List.Item.Meta
            title={<a href={`/content/{${item.post_id}}`}>{item.title}</a>}
            description={`#${item.burrow_id} 洞主`}
          />
          <div>Tag: {item.tag != undefined && showtag(item.tag)}</div>
        </List.Item>
      )}
    />
  );
}
