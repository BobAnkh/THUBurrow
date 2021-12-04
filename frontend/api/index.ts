import axios from 'axios';

axios.defaults.baseURL = process.env.API_URL;
axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

const fetcher = (url: string) => axios.get(url).then((res) => res.data);
