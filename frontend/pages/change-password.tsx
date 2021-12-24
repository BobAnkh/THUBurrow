import React, { useState } from 'react';
import 'antd/dist/antd.css';
import styles from '../styles/register.module.css';
import { LockOutlined } from '@ant-design/icons';
import { Form, Input, Divider, Button, message, Select } from 'antd';
import axios, { AxiosError } from 'axios';
import CryptoJS from 'crypto-js';
import { useRouter } from 'next/router';
import FindbackPassword from '../components/findback-password';
import { NextPage } from 'next';
import LoginForm from '../components/login-form';
import Register from '../components/register';

const { Option } = Select;
axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

type Iprops = {
  switchform: any;
};

const validate_password = /^(?![0-9]+$)(?![a-zA-Z]+$)[0-9A-Za-z-_]{6,20}$/;

export function Changepassword({ switchform }: Iprops) {
  const router = useRouter();
  const toggleForm = () => {
    switchform('findback');
  };
  async function onFinish(values: any) {
    const data = {
      password: CryptoJS.MD5(values.password).toString(),
      newpassword: CryptoJS.MD5(values.newpassword).toString(),
    };
    console.log(data, values.password, values.newpassword);
    axios
      .post(`${process.env.NEXT_PUBLIC_BASEURL}/users/change`, data)
      .then(function (res) {
        message.success('重置成功');
        window.location.href = '../login';
      })
      .catch(function (e) {
        const err = e as AxiosError;
        if (err.response?.status == 400) {
          message.error(err.response.data);
        }
        if (err.response?.status == 429) {
          message.error('请求过于频繁');
        }
        if (err.response?.status == 500) {
          message.error('后端内部错误');
        }
        message.error('找回密码失败');
      });
  }

  return (
    <div className={styles.background}>
      <title>修改密码</title>
      <div className={styles.container}>
        <div className={styles.header}>
          <h4 className={styles.column}>修改密码</h4>
        </div>
        <div className={styles.content}>
          <Form
            name='normal_rt'
            initialValues={{ remember: true }}
            onFinish={onFinish}
          >
            <Form.Item>
              <span className={styles.loginformback}>
                <a
                  onClick={() => {
                    router.push('/home');
                  }}
                >
                  返回首页
                </a>
                <Divider type='vertical' />
                <a
                  onClick={() => {
                    router.push('/profile');
                  }}
                >
                  返回个人主页
                </a>
              </span>
            </Form.Item>

            <Form.Item
              name='password'
              rules={[
                {
                  required: true,
                  message: '请输入原密码!',
                },
                {
                  pattern: validate_password,
                  message: '请输入字母和数字的6到20位组合',
                },
              ]}
            >
              <Input.Password
                prefix={<LockOutlined className='site-form-item-icon' />}
                type='password'
                placeholder='原密码'
              />
            </Form.Item>

            <Form.Item
              name='newpassword'
              rules={[
                {
                  required: true,
                  message: '请在此输入新密码!',
                },
                {
                  pattern: validate_password,
                  message: '请输入字母和数字的6到20位组合',
                },
              ]}
              hasFeedback
            >
              <Input.Password
                prefix={<LockOutlined className='site-form-item-icon' />}
                type='newpassword'
                placeholder='密码'
              />
            </Form.Item>

            <Form.Item
              name='confirm'
              dependencies={['newpassword']}
              hasFeedback
              rules={[
                {
                  required: true,
                  message: '请再次确认你的密码',
                },
                ({ getFieldValue }) => ({
                  validator(_, value) {
                    if (!value || getFieldValue('newpassword') === value) {
                      return Promise.resolve();
                    }
                    return Promise.reject(new Error('两次密码不一致'));
                  },
                }),
              ]}
            >
              <Input.Password
                prefix={<LockOutlined className='site-form-item-icon' />}
                type='confirmpassword'
                placeholder='请再次输入密码'
              />
            </Form.Item>

            <Form.Item>
              <span className={styles.loginformforgot} onClick={toggleForm}>
                {' '}
                忘记账号/密码
              </span>
            </Form.Item>

            <Form.Item>
              <Button
                type='primary'
                htmlType='submit'
                className={styles.loginformbutton}
              >
                更改密码
              </Button>
            </Form.Item>
          </Form>
        </div>
      </div>
    </div>
  );
}

const ChangePassword: NextPage = () => {
  const [FormType, settype] = useState('change');
  function switchForm(data: string) {
    settype(data);
  }
  function Switch() {
    if (FormType === 'change') {
      return <Changepassword switchform={switchForm}></Changepassword>;
    }
    if (FormType === 'findback') {
      return (
        <FindbackPassword
          switchform={switchForm}
          returnmsg='返回修改密码'
          switchmsg='change'
        ></FindbackPassword>
      );
    }
    if (FormType === 'login') {
      return <LoginForm switchform={switchForm}></LoginForm>;
    }
    if (FormType === 'register') {
      return <Register switchform={switchForm}></Register>;
    }
  }
  return <div>{Switch()}</div>;
};
export default ChangePassword;
