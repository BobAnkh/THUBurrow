import React, { ChangeEvent, Component } from 'react';
import { Form, Input, Button, Checkbox, message } from 'antd';
import { UserOutlined, LockOutlined } from '@ant-design/icons';
import 'antd/dist/antd.css';
import styles from './Register.module.css';
import CryptoJS from 'crypto-js';
import Config from '../config.json';
//加密
interface Iprops {
  switchform: any;
}
class LoginForm extends Component<Iprops> {
  constructor(prop: any) {
    super(prop);
    this.toggleForm = this.toggleForm.bind(this);
  }

  public onFinish = async (values: any) => {
    const data = {
      username: values.username,
      password: CryptoJS.MD5(values.password).toString(),
    };
    try {
      const res = await fetch(`${Config.url}/users/login`, {
        method: 'POST',
        body: JSON.stringify(data),
      });

      if (res.status === 200) {
        message.success('登录成功');
        window.location.href = '../';
      } else {
        const json = await res.json();
        message.error('登录失败');
        alert(json.errors);
      }
    } catch (e) {
      message.error('登录失败');
    }
  };

  public toggleForm = () => {
    this.props.switchform('register');
  };
  public toggleForm1 = () => {
    this.props.switchform('Findback');
  };
  public render() {
    return (
      <div className={styles.background}>
        <div className={styles.containerlogin}>
          <div className={styles.header}>
            <h4 className={styles.column}>登录</h4>
          </div>
          <div className={styles.content}>
            <Form
              name='normal_login'
              initialValues={{ remember: true }}
              onFinish={this.onFinish}
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
                <Form.Item
                  name={styles.remember}
                  valuePropName='checked'
                  noStyle
                >
                  <Checkbox className={styles.loginformremeber}>
                    记住账号
                  </Checkbox>
                </Form.Item>
                <span
                  className={styles.loginformforgot}
                  onClick={this.toggleForm1}
                >
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
                  或即刻 <a onClick={this.toggleForm}> 注册</a>
                </h4>
              </Form.Item>
            </Form>
          </div>
        </div>
      </div>
    );
  }
}

export default LoginForm;
