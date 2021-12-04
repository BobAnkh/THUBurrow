import React, { useState } from 'react';
import {
  Button,
  Checkbox,
  Form,
  Input,
  message,
  Select,
  Typography,
} from 'antd';
import CryptoJS from 'crypto-js';
import { validatePassword } from '../common/password';
import { emailSuffix } from '../common/email';
import useCountDown from 'react-countdown-hook';
import axios from 'axios';
import { useRouter } from 'next/router';

interface RegisterProps {
  switchForm: (form: string) => void;
}

const Register: React.FC<RegisterProps> = (props) => {
  const { switchForm } = props;
  const router = useRouter();
  const [form] = Form.useForm();
  const [validateEmail, setValidateEmail] = useState(emailSuffix[0]);

  const onEmailSuffixChange = (value: string) => {
    setValidateEmail(
      emailSuffix.find((e) => e.value === value) ?? emailSuffix[0]
    );
  };

  const emailSuffixSelector = (
    <Form.Item name='email-suffix' noStyle>
      <Select onChange={onEmailSuffixChange} options={emailSuffix} />
    </Form.Item>
  );

  const [sendCodeTimeLeft, { start: sendCodeTimerStart }] = useCountDown(0);
  const isSendCodeTimerDone = sendCodeTimeLeft <= 0;

  const sendCode = async () => {
    await axios.post('/users/verification', {
      email: `${form.getFieldValue('email')}${validateEmail.label}`,
    });
    sendCodeTimerStart(60000);
  };

  const emailSuffixCode = isSendCodeTimerDone ? (
    <Button onClick={sendCode}>发送验证码</Button>
  ) : (
    <Button loading>{`重新获取 ${sendCodeTimeLeft / 1000}s`}</Button>
  );

  const onFinish = async (values: {
    username: string;
    password: string;
    email: string;
    code: string;
  }) => {
    const data = {
      username: values.username,
      password: CryptoJS.MD5(values.password).toString(),
      email: `${values.email}${validateEmail.label}`,
      verification_code: values.code,
    };
    try {
      await axios.post(`/users/sign-up`, data);
      message.success('注册成功');
      router.push('/home');
    } catch (e) {
      message.error('注册失败');
    }
  };

  return (
    <Form form={form} name='register' onFinish={onFinish} scrollToFirstError>
      <Form.Item
        name='username'
        label='用户名'
        rules={[
          {
            required: true,
            message: '请输入您的用户名！',
          },
        ]}
      >
        <Input placeholder='Username' />
      </Form.Item>
      <Form.Item
        name='password'
        label='密码'
        hasFeedback
        rules={[
          { required: true, message: '请输入您的密码！' },
          {
            pattern: validatePassword,
            message: '请输入包含字母和数字的6到20位组合',
          },
        ]}
      >
        <Input.Password placeholder='Password' />
      </Form.Item>
      <Form.Item
        name='password_confirm'
        label='确认密码'
        hasFeedback
        dependencies={['password']}
        rules={[
          { required: true, message: '请再次输入您的密码以确认！' },
          ({ getFieldValue }) => ({
            validator(_, value) {
              if (!value || getFieldValue('password') === value) {
                return Promise.resolve();
              }
              return Promise.reject(new Error('两次密码不一致!'));
            },
          }),
        ]}
      >
        <Input.Password placeholder='Password' />
      </Form.Item>
      <Form.Item
        name='email'
        label='邮箱'
        rules={[
          { required: true, message: '请输入您的邮箱！' },
          {
            pattern: validateEmail.pattern,
            message: '请输入合法的邮箱！',
          },
        ]}
      >
        <Input addonAfter={emailSuffixSelector} />
      </Form.Item>
      <Form.Item name='code' label='邮箱验证码'>
        <Input addonAfter={emailSuffixCode} />
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
      >
        <Checkbox>
          注册即代表同意 <a href=''>服务条款</a>
        </Checkbox>
      </Form.Item>
      <Form.Item>
        <Button type='primary' htmlType='submit' block>
          注册
        </Button>
      </Form.Item>
      <Form.Item>
        <Typography.Text type='secondary'>
          或即刻
          <Button type='link' onClick={() => switchForm('login')}>
            登录
          </Button>
        </Typography.Text>
      </Form.Item>
    </Form>
  );
};

export default Register;
