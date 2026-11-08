# 初始化参考材料

该目录只保存可复制示例与 SDK 安装参考，不作为业务项目工作区，不在这里开发插件。

在工具包外使用 `asapflow plugin init-project --root <项目目录> --source <工具包>/content` 初始化独立项目。项目根目录包含 backend/、frontend/、locales/、docs/dev-kit/ 和 docs/project/。

需要示例时将 examples/ 下的指定插件复制到外部项目正式目录。后端 SDK 可从 runtime-sdk/ 安装；不要把它的源码副本纳入业务项目。前端 SDK 通过 npm 安装。

项目 Git 由开发者管理；工具更新只作用于工具仓库。项目文档使用 init-project --check 和 --update 更新。
