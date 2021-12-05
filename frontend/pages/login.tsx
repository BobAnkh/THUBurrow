import React, { useState } from 'react';
import type { NextPage } from 'next';
import LoginForm from '../components/login-form';
import FindbackPassword from '../components/findback-password';
import Register from '../components/register';

const Login: NextPage = () => {
  const [FormType, settype] = useState('login');
  function switchForm(data: string) {
    settype(data);
  }
  function Switch() {
    if (FormType === 'login') {
      return <LoginForm switchform={switchForm}></LoginForm>;
    }
    if (FormType === 'register') {
      return <Register switchform={switchForm}></Register>;
    }
    if (FormType === 'findback') {
      return <FindbackPassword switchform={switchForm}></FindbackPassword>;
    }
  }
  return <div>{Switch()}</div>;
};
export default Login;
