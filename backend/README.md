# Backend

## Sample

食用指南：

1. 安装rust、docker、docker-compose，安装rust-analyser插件和BetterToml插件
2. 请保证vscode打开的文件夹是`backend`而不是整体`Thuburrow`文件夹，以方便rust-analyser插件工作
3. 运行`docker-compose -f docker-compose-postgres.yml up -d`命令运行postgres数据库，请自行研究配置和pgadmin使用（访问127.0.0.1:5050可见）
4. 运行`cargo run`，或安装`cargo-watch`之后运行`cargo watch -x run`，即可运行后端
5. 使用apifox或者curl发送请求，确认结果
