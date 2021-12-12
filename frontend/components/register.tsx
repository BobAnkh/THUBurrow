import React, { useState, useEffect } from 'react';
import styles from '../styles/register.module.css';
import 'antd/dist/antd.css';
import { Form, Input, Select, Row, Col, Checkbox, Button, message } from 'antd';
import axios from 'axios';

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

export default function Register({ switchform }: Iprops) {
  const [userName, setUserName] = useState('');
  const [passWord, setPassWord] = useState('');
  const [email, setEmail] = useState('');
  const [suffix, setSuffix] = useState('@mails.tsinghua.edu.cn');
  const [btnBool, setbtnBool] = useState(false);
  const [btnText, setbtnText] = useState('发送验证码');
  const [verCode, setVerCode] = useState('');

  async function sendCode() {
    let maxTime = 5;
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
    console.log(data);
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/users/email`,
        data
      );
      var json = await res.data;
      if (json.error) {
        message.success('发送验证码成功');
      } else {
        message.error('发送验证码失败');
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

  function handleVerCode(event: any) {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      setVerCode(value);
    }
  }

  async function onFinish() {
    const data = {
      username: userName,
      password: passWord,
      email: email + suffix,
      verification_code: verCode,
    };
    try {
      const res = await axios.post(
        `${process.env.NEXT_PUBLIC_BASEURL}/6381347`,
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
                    name='Username'
                  />
                </Col>
              </Row>
            </Form.Item>
            <Form.Item
              name='password'
              rules={[
                { required: true, message: 'Please input your password!' },
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
                    name='Password'
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
