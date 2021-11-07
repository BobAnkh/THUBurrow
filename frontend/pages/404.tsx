import type { NextPage } from 'next';
import { Result, Button } from 'antd';
import Link from 'next/link';
import '../node_modules/antd/dist/antd.css';

const NotFoundPage: NextPage = () => {
  return (
    <Result
      status='404'
      title='404'
      subTitle='未找到您要访问的页面'
      extra={
        <Button type='primary'>
          <Link href='/'> 返回主页</Link>
        </Button>
      }
    />
  );
};

export default NotFoundPage;
