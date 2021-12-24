import React, { useState } from 'react';
import 'antd/dist/antd.css';
import { useRouter } from 'next/router';
import styles from '../styles/register.module.css';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { Form, Input, Row, Col, Button, message, Select } from 'antd';
import axios, { AxiosError } from 'axios';
import CryptoJS from 'crypto-js';

const { Option } = Select;
axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

type Iprops = {
  switchform: any;
  returnmsg: string;
  switchmsg: string;
};

const validate_password = /^(?![0-9]+$)(?![a-zA-Z]+$)[0-9A-Za-z-_]{6,20}$/;

export default function FindbackPassword({
  switchform,
  returnmsg,
  switchmsg,
}: Iprops) {
  const router = useRouter();
  const [btnText, setbtnText] = useState('发送验证码');
  const [btnBool, setbtnBool] = useState(false);
  const [email, setEmail] = useState('');
  const [suffix, setSuffix] = useState('@mails.tsinghua.edu.cn');

  async function onFinish(values: any) {
    const data = {
      email: email + suffix,
      password: CryptoJS.MD5(values.password).toString(),
      verification_code: values.code,
    };
    axios
      .post(`${process.env.NEXT_PUBLIC_BASEURL}/users/reset`, data)
      .then(function (res) {
        message.success('重置成功');
        window.location.href = '../login';
      })
      .catch(function (e) {
        const err = e as AxiosError;
        if (err.response?.status == 400) {
          message.error(err.response.data.code);
        }
        if (err.response?.status == 401) {
          message.error('请先登录！');
          router.push('/login');
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

  async function SendCode() {
    let maxTime = 60;
    const timer = setInterval(() => {
      if (maxTime > 0) {
        --maxTime;
        setbtnBool(true);
        setbtnText('重新获取' + maxTime);
      } else {
        setbtnBool(false);
        setbtnText('发送验证码');
        clearInterval(timer);
      }
    }, 1000);
    const data = {
      email: email + suffix,
    };
    axios
      .post(`${process.env.NEXT_PUBLIC_BASEURL}/users/reset/email`, data)
      .then(function (res) {
        message.success('发送验证码成功');
      })
      .catch(function (e) {
        const err = e as AxiosError;
        if (err.response?.status == 400) {
          message.error('邮箱格式错误或尚未注册过');
        }
        if (err.response?.status == 429) {
          message.error('请求次数过多或邮箱不存在');
        }
        if (err.response?.status == 500) {
          message.error('后端内部错误');
        }
        message.error('发送验证码失败');
      });
  }

  function handleEmail(event: any) {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setEmail(value);
    }
  }

  function handleSuffix(value: any) {
    setSuffix(value);
  }

  const selectAfter = (
    <Select
      defaultValue='@mails.tsinghua.edu.cn'
      onChange={handleSuffix}
      className={styles.select_after}
    >
      <Option value='@pku.edu.cn'>@pku.edu.cn</Option>
      <Option value='@mail.tsinghua.edu.cn'>@mail.tsinghua.edu.cn</Option>
      <Option value='@tsinghua.edu.cn'>@tsinghua.edu.cn</Option>
      <Option value='@mails.tsinghua.edu.cn'>@mails.tsinghua.edu.cn</Option>
    </Select>
  );
  return (
    <div className={styles.background}>
      <title>找回密码</title>
      <div className={styles.container}>
        <div className={styles.header}>
          <h4 className={styles.column}>找回密码</h4>
        </div>
        <div className={styles.content}>
          <Form
            name='normal_rt'
            initialValues={{ remember: true }}
            onFinish={onFinish}
          >
            <Form.Item>
              <span
                className={styles.loginformback}
                onClick={() => {
                  switchform(switchmsg);
                }}
              >
                {' '}
                {/* 返回登陆 */}
                {returnmsg}
              </span>
            </Form.Item>
            <Form.Item
              name='email'
              rules={[
                {
                  required: true,
                  message: 'Please input a valid email address!',
                },
                {
                  pattern: /^[0-9a-z-A-Z-]{1,}$/,
                  message:
                    "the address should only contains letters, numbers and '-'",
                },
              ]}
              className={styles.formStyle}
            >
              <Input
                addonAfter={selectAfter}
                onChange={(event) => handleEmail(event)}
                prefix={<UserOutlined className='site-form-item-icon' />}
                placeholder='请输入你的邮箱:'
              />
            </Form.Item>

            <Form.Item
              name='code'
              rules={[
                { required: true, message: '请输入10位验证码!', len: 10 },
              ]}
            >
              <Row gutter={13}>
                <Col span={16}>
                  <Input
                    prefix={<UserOutlined className='site-form-item-icon' />}
                    placeholder='验证码'
                  />
                </Col>
                <Col span={8}>
                  <Button
                    className={styles.loginformforgot}
                    onClick={SendCode}
                    disabled={btnBool}
                  >
                    {btnText}
                  </Button>
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='password'
              rules={[
                {
                  required: true,
                  message: '请在此输入你的密码!',
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
                type='password'
                placeholder='密码'
              />
            </Form.Item>

            <Form.Item
              name='confirm'
              dependencies={['password']}
              hasFeedback
              rules={[
                {
                  required: true,
                  message: '请再次确认你的密码',
                },
                ({ getFieldValue }) => ({
                  validator(_, value) {
                    if (!value || getFieldValue('password') === value) {
                      return Promise.resolve();
                    }
                    return Promise.reject(new Error('两次密码不一致'));
                  },
                }),
              ]}
            >
              <Input.Password
                prefix={<LockOutlined className='site-form-item-icon' />}
                type='password'
                placeholder='请再次输入密码'
              />
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
