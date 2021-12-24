import '../../node_modules/antd/dist/antd.css';
import { message } from 'antd';
import axios from 'axios';

axios.defaults.withCredentials = true;

export default function UploadImage(setNewURL: any) {
  const LIMIT_SIZE = 500 * 1000; //图片最大500kb
  const MAX_WIDTH = 2000;
  const MAX_HEIGHT = 2000;
  //const [newURL, setNewURL] = useState('');
  let compressCount = 0;

  function compress(base64: any, quality: number, mimeType: string) {
    console.log(base64);
    let canvas = document.createElement('canvas');
    let img = document.createElement('img');
    img.crossOrigin = 'anonymous';
    return new Promise<string>((resolve, reject) => {
      img.src = base64;
      img.onload = () => {
        let targetWidth, targetHeight;
        targetWidth = img.width;
        targetHeight = img.height;
        if (img.width > MAX_WIDTH) {
          targetWidth = MAX_WIDTH;
          targetHeight = (img.height * MAX_WIDTH) / img.width;
        }
        if (img.height > MAX_HEIGHT) {
          targetWidth = (img.width * MAX_HEIGHT) / img.height;
          targetHeight = MAX_HEIGHT;
        }
        canvas.width = targetWidth;
        canvas.height = targetHeight;
        let ctx = canvas.getContext('2d');
        ctx!.clearRect(0, 0, targetWidth, targetHeight); // 清除画布
        ctx!.drawImage(img, 0, 0, canvas.width, canvas.height);
        let imageData = canvas.toDataURL(mimeType, quality / 100); // 设置图片质量
        if (imageData.length - (imageData.length / 8) * 2 > LIMIT_SIZE) {
          compressCount += 1;
          compress(imageData, 0.9 - compressCount * 0.1, 'image/jpeg');
        } else {
          compressCount = 0;
          resolve(imageData);
        }
      };
    });
  }

  // 转化为Uint8Array
  function dataUrlToBlob(base64: string) {
    let bytes = window.atob(base64.split(',')[1]);
    let ab = new ArrayBuffer(bytes.length);
    let ia = new Uint8Array(ab);
    for (let i = 0; i < bytes.length; i++) {
      ia[i] = bytes.charCodeAt(i);
    }
    return ia;
  }

  const upLoadToServer = async (bytes: Uint8Array, type: string) => {
    try {
      console.log('size', bytes.length);
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/storage/images`,
        bytes,
        { headers: { 'Content-Type': type } }
      );
      setNewURL(`${res.data}`);
    } catch (e) {
      message.error('上传图片失败！');
    }
  };

  const uploadImage = (event: any) => {
    const reader = new FileReader();
    console.log('step0 done!');
    reader.onload = async function (event) {
      let compressedDataURL = await compress(
        event.target?.result,
        90,
        'image/jpeg'
      );
      let compressedImageUint8 = dataUrlToBlob(compressedDataURL);
      upLoadToServer(compressedImageUint8, 'image/jpeg');
    };
    reader.readAsDataURL(event.target.files[0]);
  };

  return <input type='file' accept='image/*' onChange={uploadImage} />;
}
