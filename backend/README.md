# Backend

## Sample

食用指南：

1. 安装rust、docker、docker-compose，安装rust-analyser插件和BetterToml插件
2. 请保证vscode打开的文件夹是`backend`而不是整体`Thuburrow`文件夹，以方便rust-analyser插件工作
3. 运行`docker-compose -f docker-compose-postgres.yml up -d`命令运行postgres数据库，请自行研究配置和pgadmin使用（访问127.0.0.1:5050可见）
4. 运行`cargo run`，或安装`cargo-watch`之后运行`cargo watch -x run`，即可运行后端
5. 使用apifox或者curl发送请求，确认结果

## Convention

后端开发请注意相关规范：

1. 注意从`backend`分支checkout出一个新分支开发你的功能或修复bug，分支名称尽量与功能相关
2. Rust编写规范：[coding-style](https://wiki.jikexueyuan.com/project/rust-primer/coding-style/style.html)，主要注意其中的命名规范和注释规范
3. 编写完代码提交前，请运行`cargo clippy`和`cargo fmt`进行格式规范和代码优化校验，`cargo fmt`也可由vscode中右键菜单里的`Format Document`(`格式化文档`)来代替
