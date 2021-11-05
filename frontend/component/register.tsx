import React from "react";
import styles from "./register.module.css"
import "antd/dist/antd.css"
import CryptoJS from "crypto-js";
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
  import { SettingOutlined, UserOutlined, LockOutlined, EyeInvisibleOutlined, EyeTwoTone, AudioOutlined} from '@ant-design/icons';
import {thisExpression } from '@babel/types';

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
  

  interface Iprops{
    switchform:any,
}

class Register extends React.Component<Iprops,any>{
    constructor(props:any){
        super(props);
        this.state = {usrname:null,
                      password:null,
                      email:null,
                      btnText:'发送验证码',
                      btnBool:false,
                      suffix:"@mails.tsinghua.edu.cn"
                    };
        this.sendCode = this.sendCode.bind(this);
        this.onFinish = this.onFinish.bind(this);
    };

    handleSuffix = (value: any) => {
        console.log("You enter this!");
        this.setState(()=>({suffix: value}));
    }   
    
    
    selectAfter = (
        <Select defaultValue="@mails.tsinghua.edu.cn" onChange={this.handleSuffix} className={styles.select_after}>
            <Option value="@pku.edu.cn">@pku.edu.cn</Option>
            <Option value="@mails.tsinghua.edu.cn">@mails.tsinghua.edu.cn</Option>
        </Select>
    );

    public toggleForm=()=>{
        this.props.switchform("login")
    }

    handleUsrName = (event:any,) => {
        if(event && event.target && event.target.value)
        {
            let value = event.target.value;
            this.setState(()=>({usrname: value}));
        }
    }
    
    handlePassWord = (event: any) => {
        if(event && event.target && event.target.value)
        {
            let value = event.target.value;
            this.setState(()=>({password: value}));
        }
    }

    handleEmail = (event: any) => {
        if(event && event.target && event.target.value)
        {
            let value = event.target.value;
            this.setState(()=>({email: value}));
        }
    }

    SendCode() {
            let maxTime=60
            const timer = setInterval(() => {
             if (maxTime > 0) {
               --maxTime
               this.setState({
                 btnText: '重新获取' + maxTime,
                 btnBool: true
               })
             }
             else {
               this.setState({
                 btnText: '发送验证码',
                 btnBool: false
               })
             }
           }, 1000)

     }
   
    onFinish = async(values:any)=>{
            const data = {
                username:values.username,
                 password:CryptoJS.MD5(values.password).toString(),
            }
            //console.log(data)
            try{
                const res = await fetch("http://127.0.0.1:4523/mock2/435762/6381347",{
                method:'POST',
                body:JSON.stringify(data),
            })
            
            if (res.status===200){
                const json = await res.json();
             //   localStorage.setItem(json,"value")
                message.success("登录成功");
                console.log('success',json)
                window.location.href='../Login'
            }
            else{
                const json = await res.json(); 
                message.error("登录失败");
                console.log(json)
                alert(json.error)       
            }
            }catch(e){
            message.error("登录失败");
            }
        }
    
    sendCode = () => console.log('This is :', this.state.usrname, this.state.password, this.state.email+this.state.suffix);  

    render(){
        return(
            <div className={styles.background}>
                <div className={styles.container}>
                    <div className={styles.header}>
                        <h4 className={styles.column}>注册</h4>
                    </div>
                    <div className={styles.content}>
                        <Form 
                            name="register"
                            onFinish={this.onFinish}
                            >
                        <Form.Item
                            name="username"
                            rules={[{ required: true, message: 'Please input your Username!' }]}
                            className={styles.formStyle}
                        >
                            <Row>
                                <Col span={6}>用户名：</Col>
                                <Col span={18}><Input placeholder="Username" onChange = {(event)=> this.handleUsrName(event)} className={styles.inputBox}/></Col>
                            </Row>
                            
                        </Form.Item>
                        <Form.Item
                            name="password"
                            rules={[{ required: true, message: 'Please input your password!' },
                            ({ getFieldValue }) => ({
                                validator(role, value){
                                    let password_value = getFieldValue('password_confirm');
                                    if(password_value && value !== password_value)
                                        return Promise.reject("两次输入的密码不一致")
                                    return Promise.resolve();
                                }
                            })
                            ]
                        }
                            className={styles.formStyle}
                        >
                            <Row>
                                <Col span={6}>密码:</Col>
                                <Col span={18}> 
                                    <Input.Password placeholder="password" onChange = {(event)=> this.handlePassWord(event)} className={styles.inputBox}/>
                                </Col>
                            </Row>
                            
                        </Form.Item>
                        <Form.Item
                            name="password_confirm"
                            rules={[{ required: true, message: 'Please input your password!' },
                            // ({get)      
                            
                        
                                ]}
                            className={styles.formStyle}
                        >
                            <Row >
                                <Col span={6}>确认密码:</Col>
                                <Col span={18}>
                                    <Input.Password placeholder="password" className={styles.inputBox}/>
                                </Col>
                            </Row>
                        </Form.Item>
                        {/* <Space direction="vertical"> */}
                            <Form.Item
                                name="email"
                                rules={[{ required: true, message: 'Please input a valid email address!'},
                                        { pattern: /^[0-9a-z-A-Z-]{1,}$/, message: "the address should only contains letters, numbers and '-'"}
                                        ]}
                                className={styles.formStyle}
                            >
                                <Row >
                                    <Col span={6}>邮箱:</Col>
                                    <Col span={18}><Input addonAfter={this.selectAfter} onChange = {(event)=> this.handleEmail(event)} className={styles.inputBox}/></Col>
                                </Row>
                            
                            </Form.Item>
                        <Form.Item>
                            <Row >
                                <Col span={6}>邮箱验证码:</Col>
                                <Col span={12}><Input/></Col>
                                <Col span={6}><Button type='primary' onClick={this.SendCode.bind(this)} disabled={this.state.btnBool}>{this.state.btnText}</Button>
                                </Col>
                            </Row>
                        </Form.Item>


                        <Form.Item
                            name="agreement"
                            valuePropName="checked"
                            rules={[
                            {
                                validator: (_, value) =>
                                value ? Promise.resolve() : Promise.reject(new Error('Should accept agreement')),
                            },
                            ]}
                            {...tailFormItemLayout}

                        >
                            <Checkbox>
                            注册即代表同意 <a href="">服务条款</a>
                            </Checkbox>
                        </Form.Item>




                        <Form.Item>
                            <Button type="primary" htmlType="submit" className="login-form-button" block>
                            Register
                            </Button>
                        </Form.Item>
                        <h4 >
                            或即刻 <a  onClick={this.toggleForm}> 登录</a>
                        </h4>
                        </Form>
                    </div>
                </div>
            </div>
        )
    }
}


export default Register