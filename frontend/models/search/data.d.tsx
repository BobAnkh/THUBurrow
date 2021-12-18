export interface PostListItemDataType {
  post_id: number;
  title: string;
  update_time: string;
  burrow_id: number;

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
export interface Postreply {
  reply_id: number;
  post_id: number;
  burrow_id: number;
  content: string;
  update_time: string;
  create_time: string;
}
export interface Reply {
  post_id: number;
  replies: [
    {
      reply_id: number;
      post_id: number;
      burrow_id: number;
      content: string;
      update_time: string;
    }
  ];
}
export interface BurrowListItemDataType {
  burrow_id: number;
  title: string;
  description: string;
  update_time: string;

  status?: boolean;
  star?: number;
  like?: number;

  highlights?: [{ field: string; snippet: string }];
}
export interface BurrowDataType {
  post_id: number;
  burrow_id: number;
  title: string;
  section: [];
  tag: [];

  like_num: number;
  collection_num: number;
  post_len: number;

  create_time: string;
  updata_time: string;
}
