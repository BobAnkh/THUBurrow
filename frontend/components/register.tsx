import React, { useState, useEffect } from 'react';
import styles from '../styles/register.module.css';
import 'antd/dist/antd.css';
import { Form, Input, Select, Row, Col, Checkbox, Button, message } from 'antd';
import axios from 'axios';
import CryptoJS from 'crypto-js';

axios.defaults.withCredentials = true;
axios.defaults.headers.post['Content-Type'] = 'application/json';

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

const validate_password = /^(\w){8,20}$/;

export default function Register({ switchform }: Iprops) {
  const [userName, setUserName] = useState('');
  const [passWord, setPassWord] = useState('');
  const [email, setEmail] = useState('');
  const [suffix, setSuffix] = useState('@mails.tsinghua.edu.cn');
  const [btnBool, setbtnBool] = useState(false);
  const [btnText, setbtnText] = useState('发送验证码');
  const [verCode, setVerCode] = useState('');

  async function sendCode() {
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
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/users/email`,
        data
      );
      var json = await res.data;
      if (json.error) {
        message.error('发送验证码失败');
      } else {
        message.success('发送验证码成功');
      }
    } catch (e) {
      message.error('发送验证码失败');
      alert(e);
    }
  }

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
      style={{ width: '200px' }}
    >
      <Option value='@pku.edu.cn'>@pku.edu.cn</Option>
      <Option value='@mail.tsinghua.edu.cn'>@mail.tsinghua.edu.cn</Option>
      <Option value='@tsinghua.edu.cn'>@tsinghua.edu.cn</Option>
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

  function handleVerCode(event: any) {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setVerCode(value);
    }
  }

  async function onFinish() {
    const data = {
      username: userName,
      password: CryptoJS.MD5(passWord).toString(),
      email: email + suffix,
      verification_code: verCode,
    };
    try {
      console.log(data);
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/users/sign-up`,
        data
      );
      var json = await res.data;
      if (json.error) {
        message.error('注册失败');
      } else {
        message.success('注册成功');
        window.location.href = '../login';
      }
    } catch (e) {
      message.error('注册失败');
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
                { required: true, message: '请输入用户名!' },
                { pattern: /^[^\s]*$/, message: '用户名不能包含空格' },
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
                    name='Username'
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='password'
              hasFeedback
              rules={[
                { required: true, message: '请输入密码!' },
                {
                  pattern: validate_password,
                  message: '请输入字母、数字和下划线的8到20位组合',
                },
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
                    name='Password'
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='password_confirm'
              hasFeedback
              dependencies={['password']}
              rules={[
                { required: true, message: '请确认密码!' },
                ({ getFieldValue }) => ({
                  validator(_, value) {
                    if (!value || value === getFieldValue('password')) {
                      return Promise.resolve();
                    }
                    return Promise.reject(new Error('两次密码不一致'));
                  },
                }),
              ]}
              className={styles.formStyle}
            >
              <Row>
                <Col span={6}>确认密码:</Col>
                <Col span={18}>
                  <Input.Password
                    placeholder='password'
                    className={styles.inputBox}
                    name='Password_Confirm'
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='email'
              rules={[
                {
                  required: true,
                  message: '请填写有效邮箱地址!',
                },
                {
                  pattern: /^[0-9a-z-A-Z-]{1,}$/,
                  message: "地址只能是字母，数字和'-'的组合",
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
                    name='Email'
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item>
              <Row>
                <Col span={6}>邮箱验证码:</Col>
                <Col span={12}>
                  <Input
                    name='Email_Confirm'
                    onChange={(event) => handleVerCode(event)}
                  />
                </Col>
                <Col span={6}>
                  <Button
                    type='primary'
                    className={styles.send}
                    disabled={btnBool}
                    block
                    onClick={sendCode}
                  >
                    {btnText}
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
