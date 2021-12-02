export interface Params {
  id: number;
  tag: string;
  keyword: string;
  order: string;
  area: string;
}
export interface PostListItemDataType {
  document: {
    post_id: string;
    title: string;
    url: string;
    updated_time: string;
    created_time: string;
    star: number;
    like: number;
    dislike: number;
    description: [];
    message: number;
    introduction: string;
  };
  highlights?: [{ field: string; snippet: string }];
}
export interface BurrowListItemDataType {
  document: {
    burrow_id: string;
    title: string;
    url: string;
    updated_time: string;
    created_time: string;
    status: boolean; //洞是否废弃
    star: number;
    post_number: number;
    introduction: string;
  };
  highlights?: [{ field: string; snippet: string }];
}
