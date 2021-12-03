import React, { useState } from 'react';
import 'antd/dist/antd.css';
import styles from '../styles/register.module.css';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import { Form, Input, Row, Col, Button } from 'antd';

type Iprops = {
  switchform: any;
};
//密码验证
export const validate_password =
  /^(?![0-9]+$)(?![a-zA-Z]+$)[0-9A-Za-z-_]{6,20}$/;

export default function FindbackPassword({ switchform }: Iprops) {
  async function onFinish(values: any) {}
  const [btnText, setbtnText] = useState('发送验证码');
  const [btnBool, setbtnBool] = useState(false);
  const toggleForm = () => {
    switchform('login');
  };
  function SendCode() {
    let maxTime = 60;
    const timer = setInterval(() => {
      if (maxTime > 0) {
        --maxTime;
        setbtnBool(true);
        setbtnText('重新获取' + maxTime);
      } else {
        setbtnBool(false);
        setbtnText('发送验证码');
      }
    }, 1000);
  }
  return (
    <div className={styles.background}>
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
              <span className={styles.loginformback} onClick={toggleForm}>
                {' '}
                返回登陆
              </span>
            </Form.Item>
            <Form.Item
              name='username'
              rules={[
                {
                  type: 'email',
                  message: '邮箱格式不正确',
                },
                {
                  required: true,
                  message: '请输入你的邮箱!',
                },
              ]}
            >
              <Input
                prefix={<UserOutlined className='site-form-item-icon' />}
                placeholder='请输入你的邮箱:'
              />
            </Form.Item>

            <Form.Item
              name='code'
              rules={[{ required: true, message: '请输入6位验证码!', len: 6 }]}
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
