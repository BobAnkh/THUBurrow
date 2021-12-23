import React from 'react';
import { Form, Input, Button, Checkbox, message } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import 'antd/dist/antd.css';
import styles from '../styles/register.module.css';
import CryptoJS from 'crypto-js';
import axios, { AxiosError } from 'axios';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';
//加密
type Iprops = {
  switchform: any;
};
export default function LoginForm({ switchform }: Iprops) {
  async function OnFinish(values: any) {
    const data = {
      username: values.username,
      password: CryptoJS.MD5(values.password).toString(),
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/users/login`,
        data
      );
      message.success('登录成功,正在跳转到主页...');
      window.location.href = '../home';
    } catch (e) {
      const err = e as AxiosError;
      if (err.response?.status == 400) {
        message.error('用户名或密码错误');
      }
      if (err.response?.status == 500) {
        message.error('服务器错误');
      }
      message.error('登陆失败');
    }
  }
  const toggleForm = () => {
    switchform('register');
  };
  const toggleForm1 = () => {
    switchform('findback');
  };
  return (
    <div className={styles.background}>
      <title>登录</title>
      <div className={styles.containerlogin}>
        <div className={styles.header}>
          <h4 className={styles.column}>登录</h4>
        </div>
        <div className={styles.content}>
          <Form
            name='normal_login'
            initialValues={{ remember: true }}
            onFinish={OnFinish}
          >
            <Form.Item
              name='username'
              rules={[{ required: true, message: '请输入你的账号!' }]}
            >
              <Input
                type='username'
                prefix={<UserOutlined className='site-form-item-icon' />}
                placeholder='账号'
              />
            </Form.Item>
            <Form.Item
              name='password'
              rules={[
                { required: true, message: '密码不能为空!' },

                { min: 6, message: '密码太短' },
                { max: 20, message: '密码超出范围' },
              ]}
            >
              <Input
                prefix={<LockOutlined className='site-form-ite-icon' />}
                type='password'
                placeholder='密码'
              />
            </Form.Item>
            <Form.Item>
              <Form.Item name={styles.remember} valuePropName='checked' noStyle>
                <Checkbox className={styles.loginformremeber}>
                  记住账号
                </Checkbox>
              </Form.Item>
              <span className={styles.loginformforgot} onClick={toggleForm1}>
                {' '}
                忘记账号/密码
              </span>
            </Form.Item>
            <Form.Item>
              <Button
                type='primary'
                htmlType='submit'
                className='login-form-button'
                block
              >
                登录
              </Button>
              <h4>
                或即刻 <a onClick={toggleForm}> 注册</a>
              </h4>
            </Form.Item>
          </Form>
        </div>
      </div>
    </div>
  );
}
