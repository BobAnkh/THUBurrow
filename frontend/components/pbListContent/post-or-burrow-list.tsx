import React from 'react';
import moment from 'moment';
import styles from '../../styles/pb-list.module.css';

type PBListContentProps = {
  data: {
    introduction: string;
    updated_time: string;
  };
};

const PBListContent: React.FC<PBListContentProps> = ({
  data: { introduction, updated_time },
}) => (
  <div className={styles.listContent}>
    <div className={styles.description}>{introduction}</div>
    <div className={styles.extra}>
      <em>updated at: {moment(updated_time).format('YYYY-MM-DD HH:mm')}</em>
    </div>
  </div>
);

export default PBListContent;
