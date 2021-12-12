import React, { useState, useEffect } from 'react';
import styles from '../styles/register.module.css';
import 'antd/dist/antd.css';
import {
  Form,
  Input,
  InputNumber,
  Cascader,
  Select,
  Row,
  Col,
  Checkbox,
  Button,
  AutoComplete,
  Space,
  message,
} from 'antd';
import {
  SettingOutlined,
  UserOutlined,
  LockOutlined,
  EyeInvisibleOutlined,
  EyeTwoTone,
  AudioOutlined,
} from '@ant-design/icons';
import { thisExpression } from '@babel/types';
import axios from 'axios';

const { Search } = Input;

type RegisProps = {};
const { Option } = Select;

const tailFormItemLayout = {
  wrapperCol: {
    xs: {
      span: 24,
      offset: 0,
    },
    sm: {
      span: 16,
      offset: 0,
    },
  },
};

type Iprops = {
  switchform: any;
};

const validate_password = /^(?![0-9]+$)(?![a-zA-Z]+$)[0-9A-Za-z-_]{6,20}$/;

export default function Register({ switchform }: Iprops) {
  const [userName, setUserName] = useState('');
  const [passWord, setPassWord] = useState('');
  const [email, setEmail] = useState('');
  const [suffix, setSuffix] = useState('@mails.tsinghua.edu.cn');
  const [count, setCount] = useState(5);
  const [counting, setCounting] = useState(false);

  function sendCode() {}

  function handleSuffix(value: any) {
    setSuffix(value);
  }

  const ToggleForm = () => {
    switchform('login');
  };

  const selectAfter = (
    <Select
      defaultValue='@mails.tsinghua.edu.cn'
      onChange={handleSuffix}
      className={styles.select_after}
    >
      <Option value='@pku.edu.cn'>@pku.edu.cn</Option>
      <Option value='@mails.tsinghua.edu.cn'>@mails.tsinghua.edu.cn</Option>
    </Select>
  );

  function handleUsrName(event: any) {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setUserName(value);
    }
  }

  function handlePassWord(event: any) {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setPassWord(value);
    }
  }

  function handleEmail(event: any) {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setEmail(value);
    }
  }

  async function onFinish() {
    const data = {
      username: userName,
      password: passWord,
      email: email,
      verification_code: '111111',
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/users/sign-up`,
        data
      );
      message.success('注册成功');
      window.location.href = '../login';
    } catch (e) {
      message.error('登陆失败');
      alert(e);
    }
  }

  return (
    <div className={styles.background}>
      <title>注册</title>
      <div className={styles.container}>
        <div className={styles.header}>
          <h4 className={styles.column}>注册</h4>
        </div>
        <div className={styles.content}>
          <Form name='register' onFinish={onFinish}>
            <Form.Item
              name='username'
              rules={[
                { required: true, message: 'Please input your Username!' },
              ]}
              className={styles.formStyle}
            >
              <Row>
                <Col span={6}>用户名：</Col>
                <Col span={18}>
                  <Input
                    placeholder='Username'
                    onChange={(event) => handleUsrName(event)}
                    className={styles.inputBox}
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='password'
              rules={[
                { required: true, message: 'Please input your password!' },
                {
                  pattern: validate_password,
                  message: '请输入字母和数字的6到20位组合',
                },
                ({ getFieldValue }) => ({
                  validator(role, value) {
                    let password_value = getFieldValue('password_confirm');
                    if (password_value && value !== password_value)
                      return Promise.reject('两次输入的密码不一致');
                    return Promise.resolve();
                  },
                }),
              ]}
              className={styles.formStyle}
            >
              <Row>
                <Col span={6}>密码:</Col>
                <Col span={18}>
                  <Input.Password
                    placeholder='password'
                    onChange={(event) => handlePassWord(event)}
                    className={styles.inputBox}
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='password_confirm'
              rules={[
                { required: true, message: 'Please input your password!' },
              ]}
              className={styles.formStyle}
            >
              <Row>
                <Col span={6}>确认密码:</Col>
                <Col span={18}>
                  <Input.Password
                    placeholder='password'
                    className={styles.inputBox}
                  />
                </Col>
              </Row>
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
              <Row>
                <Col span={6}>邮箱:</Col>
                <Col span={18}>
                  <Input
                    addonAfter={selectAfter}
                    onChange={(event) => handleEmail(event)}
                    className={styles.inputBox}
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item>
              <Row>
                <Col span={6}>邮箱验证码:</Col>
                <Col span={12}>
                  <Input />
                </Col>
                <Col span={6}>
                  <Button
                    type='primary'
                    className={styles.send}
                    disabled={counting ? true : false}
                    block
                    onClick={sendCode}
                  >
                    {counting ? count + '秒后重发' : '获取验证码'}
                  </Button>
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='agreement'
              valuePropName='checked'
              rules={[
                {
                  validator: (_, value) =>
                    value
                      ? Promise.resolve()
                      : Promise.reject(new Error('Should accept agreement')),
                },
              ]}
              {...tailFormItemLayout}
            >
              <Checkbox>
                注册即代表同意 <a href=''>服务条款</a>
              </Checkbox>
            </Form.Item>
            <Form.Item>
              <Button
                type='primary'
                htmlType='submit'
                className='login-form-button'
                block
              >
                Register
              </Button>
            </Form.Item>
          </Form>
        </div>
      </div>
      <div className={styles.tailer}>
        已有帐号？<a onClick={ToggleForm}>现在登录</a>
      </div>
    </div>
  );
}
