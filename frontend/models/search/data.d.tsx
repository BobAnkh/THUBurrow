export interface PostListItemDataType {
  post_id: number;
  title: string;
  update_time: string;

  section: [];
  tag: [];

  created_time?: string;
  collection_num?: number;
  like_num?: number;
  dislike?: number;
  description?: [];
  message?: number;
  introduction?: string;
  highlights?: [{ field: string; snippet: string }];
}
export interface BurrowListItemDataType {
  burrow_id?: number;
  title: string;
  description: string;
  update_time: string;

  posts?: [
    {
      title: string;
      like_num: string;
      collection_num: string;
      post_len: string;
      updata_time: string;
      tag: [];
    }
  ];

  status?: boolean;
  star?: number;
  like?: number;

  highlights?: [{ field: string; snippet: string }];
}
