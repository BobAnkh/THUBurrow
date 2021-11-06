import React, { Component } from 'react';
import LoginForm from '../components/login-form';
import FindbackPassword from '../components/findback-password';
import Register from '../components/register';
interface IState {
  FormType: string;
}
class Login extends Component<IState> {
  public constructor(props: any) {
    super(props);
    this.state = {
      FormType: 'login',
    };
  }

  public readonly state: Readonly<IState> = {
    FormType: 'login',
  };

  public switchForm = (date: any) => {
    this.setState({
      FormType: date,
    });
  };
  public Switch() {
    if (this.state.FormType === 'login') {
      return <LoginForm switchform={this.switchForm}></LoginForm>;
    }
    if (this.state.FormType === 'register') {
      return <Register switchform={this.switchForm}></Register>;
    }
    if (this.state.FormType === 'Findback') {
      return <FindbackPassword switchform={this.switchForm}></FindbackPassword>;
    }
  }
  public render() {
    return (
      <div>
        <div>{this.Switch()}</div>
      </div>
    );
  }
}
export default Login;
