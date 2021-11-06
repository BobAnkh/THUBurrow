import React from 'react';
import styles from './register.module.css';
import 'antd/dist/antd.css';
import CryptoJS from 'crypto-js';
import Config from '../config.json';
import { validate_password } from './findbackPassword';
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

interface Iprops {
  switchform: any;
}

class Register extends React.Component<Iprops, any> {
  constructor(props: any) {
    super(props);
    this.state = {
      usrname: null,
      password: null,
      email: null,
      btnText: '发送验证码',
      btnBool: false,
      suffix: '@mails.tsinghua.edu.cn',
    };
    this.sendCode = this.sendCode.bind(this);
    this.onFinish = this.onFinish.bind(this);
  }

  handleSuffix = (value: any) => {
    console.log('You enter this!');
    this.setState(() => ({ suffix: value }));
  };

  selectAfter = (
    <Select
      defaultValue='@mails.tsinghua.edu.cn'
      onChange={this.handleSuffix}
      className={styles.select_after}
    >
      <Option value='@pku.edu.cn'>@pku.edu.cn</Option>
      <Option value='@mails.tsinghua.edu.cn'>@mails.tsinghua.edu.cn</Option>
    </Select>
  );

  public toggleForm = () => {
    this.props.switchform('login');
  };

  handleUsrName = (event: any) => {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      this.setState(() => ({ usrname: value }));
    }
  };

  handlePassWord = (event: any) => {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      this.setState(() => ({ password: value }));
    }
  };

  handleEmail = (event: any) => {
    if (event && event.target && event.target.value) {
      let value = event.target.value;
      this.setState(() => ({ email: value }));
    }
  };

  SendCode() {
    let maxTime = 60;
    const timer = setInterval(() => {
      if (maxTime > 0) {
        --maxTime;
        this.setState({
          btnText: '重新获取' + maxTime,
          btnBool: true,
        });
      } else {
        this.setState({
          btnText: '发送验证码',
          btnBool: false,
        });
      }
    }, 1000);
  }

  onFinish = async (values: any) => {
    const data = {
      username: values.username,
      password: CryptoJS.MD5(values.password).toString(),
      email: values.email,
      code: values.code,
    };
    try {
      const res = await fetch(`${Config.url1}/6381347`, {
        method: 'POST',
        body: JSON.stringify(data),
      });
      if (res.status === 200) {
        message.success('注册成功');
        window.location.href = '../Login';
      } else {
        const json = await res.json();
        console.log(json);
        message.error('注册失败');
        alert(json.errors);
      }
    } catch (e) {
      message.error('注册失败');
    }
  };

  sendCode = () =>
    console.log(
      'This is :',
      this.state.usrname,
      this.state.password,
      this.state.email + this.state.suffix
    );

  render() {
    return (
      <div className={styles.background}>
        <div className={styles.container}>
          <div className={styles.header}>
            <h4 className={styles.column}>注册</h4>
          </div>
          <div className={styles.content}>
            <Form name='register' onFinish={this.onFinish}>
              <Form.Item
                name='username'
                rules={[{ required: true, message: '请输入你的用户名!' }]}
                className={styles.formStyle}
              >
                <Row>
                  <Col span={6}>用户名：</Col>
                  <Col span={18}>
                    <Input
                      placeholder='Username'
                      onChange={(event) => this.handleUsrName(event)}
                      className={styles.inputBox}
                    />
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
                className={styles.formStyle}
              >
                <Row>
                  <Col span={6}>密码:</Col>
                  <Col span={18}>
                    <Input.Password
                      placeholder='password'
                      onChange={(event) => this.handlePassWord(event)}
                      className={styles.inputBox}
                    />
                  </Col>
                </Row>
              </Form.Item>
              <Form.Item
                name='password_confirm'
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
                    message: '请输入有效的邮箱',
                  },
                  {
                    pattern: /^[0-9a-z-A-Z-]{1,}$/,
                    message: '只包含字母，数组和-',
                  },
                ]}
                className={styles.formStyle}
              >
                <Row>
                  <Col span={6}>邮箱:</Col>
                  <Col span={18}>
                    <Input
                      addonAfter={this.selectAfter}
                      onChange={(event) => this.handleEmail(event)}
                      className={styles.inputBox}
                    />
                  </Col>
                </Row>
              </Form.Item>
              <Form.Item name='code'>
                <Row>
                  <Col span={6}>邮箱验证码:</Col>
                  <Col span={12}>
                    <Input />
                  </Col>
                  <Col span={6}>
                    <Button
                      type='primary'
                      onClick={this.SendCode.bind(this)}
                      disabled={this.state.btnBool}
                    >
                      {this.state.btnText}
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
                        : Promise.reject(new Error('请同意服务条款')),
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
              <h4>
                或即刻 <a onClick={this.toggleForm}> 登录</a>
              </h4>
            </Form>
          </div>
        </div>
      </div>
    );
  }
}

export default Register;
