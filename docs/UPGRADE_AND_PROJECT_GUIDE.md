# 开发工具包升级与插件项目操作说明

适用于 CLI 0.2.10 起的独立项目模式。此次发布的开发文档内容版本为 0.1.1；CLI 与文档版本分别管理。

## 1. 本次目录和职责变化

一台电脑共用一套工具包，每个项目使用自己的插件源码仓库，目录彼此独立：

```text
Asapment/
├── plugin-dev-kit/              # 平台团队维护的工具仓库，使用 git pull 更新
│   ├── bin/                     # Windows / macOS / Linux CLI
│   ├── content/                 # 带版本和校验清单的开发文档
│   └── project-template/        # 可复制示例与 SDK 安装参考
└── projects/
    ├── customer-a-plugins/      # 项目 A 的独立 Git 仓库
    └── customer-b-plugins/      # 项目 B 的独立 Git 仓库
```

工具包仓库：[zhuxueao123/plugin-dev-kit](https://github.com/zhuxueao123/plugin-dev-kit)。项目仓库地址由各项目团队自行创建和维护。

- 业务插件在 projects/ 下开发，不在工具包的 project-template/ 中开发。
- CLI 创建目录、校验、打包和更新开发文档；不执行项目的 git init、提交、拉取或远端配置。
- 工具包更新与项目源码同步是两次独立操作。更新工具包不会自动升级各项目文档，也不会自动发布插件或升级服务器。

## 2. 更新或首次获取工具包

**旧业务源码还在工具包的 plugin-workspace/ 内时，先按第 3 节备份、迁出，再更新旧工具目录。**新版已将该目录调整为 project-template/，不要依靠 Git 自动迁移业务代码。

### 已经完成分离的开发人员

在工具包目录检查并更新：

```sh
cd <工具包目录>
git status --short
git pull --ff-only
asapflow --version
```

git status 如显示未提交改动，先检查内容；不要用 reset --hard、clean 或强制覆盖来更新。工具文件被自行修改时，先另存备份，再由维护者协助合并。--ff-only 失败时先处理分支差异，不强制拉取。

### 首次使用，或旧工具目录中存在混合改动

建议将新工具包克隆到一个全新目录，保留旧目录直至迁移验收结束：

```sh
git clone https://github.com/zhuxueao123/plugin-dev-kit.git <新的工具包目录>
```

将本机对应目录加入用户 PATH：

| 系统 | PATH 中加入的目录 |
| --- | --- |
| Windows x64 | `<工具包目录>\bin\windows\asapflow` |
| macOS Apple Silicon | `<工具包目录>/bin/macos/asapflow` |
| Linux x86_64 | `<工具包目录>/bin/linux-x86_64/asapflow` |

Windows 可在“编辑账户的环境变量 → Path”中添加，重新打开终端。macOS/Linux 可在自己的 Shell 配置文件中添加对应目录。不要把 CLI 二进制复制到每个项目。

在任意目录执行 asapflow --version，确认调用的是更新后的工具。Windows 可用 Get-Command asapflow，macOS/Linux 可用 command -v asapflow 检查实际路径，避免旧版本排在 PATH 前面。

可在工具包目录执行 `python verify-dev-kit.py --mode delivery`（macOS/Linux 通常使用 python3），验证程序与文档校验值。CLI 为 GitHub Actions 预编译交付，无需开发人员自行编译 Rust。

## 3. 已有插件迁移

### 情况 A：插件已经位于工具包外，且有独立 Git 仓库

1. 确认目录内包含 backend/、frontend/、locales/ 等项目文件。
2. 备份未提交改动，记录当前分支和提交。
3. 按下面命令在该项目目录初始化开发约定和文档；保留现有 .git，不重新创建或替换仓库。

```sh
cd <已有项目目录>
asapflow plugin init-project --root . --source <工具包目录>/content --offline
```

Windows 同样可使用上述命令；含空格的路径请加双引号，路径分隔符也可使用反斜线。

### 情况 B：业务插件还在旧工具包的 plugin-workspace/ 内

迁移顺序是：**完整备份 → 复制到工具包之外 → 初始化 → 验证 → 提交项目仓库**。不要先拉取新版覆盖旧工作区。

1. 完整备份旧工具目录和 plugin-workspace/，包括未提交文件、语言资源及本机配置。备份不提交到 Git。
2. 将旧 plugin-workspace/ 的内容复制到一个尚不存在的外部项目目录。复制工作区内容即可，不要把工具包根目录的 .git、bin/ 或整套工具文档复制过去。

macOS/Linux 示例：

```sh
mkdir -p "$HOME/Asapment/projects"
# 以下目标 customer-a-plugins 必须尚不存在；已有目标先停止并人工合并。
cp -R /path/to/old-kit/plugin-workspace "$HOME/Asapment/projects/customer-a-plugins"
```

Windows PowerShell 示例：

```powershell
$ProjectParent = Join-Path $env:USERPROFILE "Asapment\projects"
$ProjectDir = Join-Path $ProjectParent "customer-a-plugins"
if (Test-Path $ProjectDir) { throw "目标目录已存在，请先人工检查，不要覆盖。" }
New-Item -ItemType Directory -Force -Path $ProjectParent | Out-Null
Copy-Item -Recurse "C:\path\to\old-kit\plugin-workspace" $ProjectDir
```

3. 如果旧工作区本身有独立 .git，一并保留；如果只是工具仓库中的普通目录，则由开发者在迁移验证后新建项目仓库。不要把工具仓库的历史误当成项目历史。
4. 检查实际业务插件是否曾写在 examples/ 内。新模式应放入 backend/<plugin-code>/ 或 frontend/src/pages/<plugin-code>/ 正式目录；同名目录已存在时人工合并，不直接覆盖。前端示例还需要项目根下完整的 frontend/package.json 和 vite.config.mjs。
5. 在新目录运行初始化：

```sh
cd <新的项目目录>
asapflow plugin init-project --root . --source <新工具包目录>/content --offline
```

6. 逐项检查：

- 现有源码、语言资源、前端配置是否完整；初始化只补缺失文件，不替换已有文件。
- 旧 AGENTS.md 是否仍引用 plugin-workspace/、工具包内的脚本或旧文档路径。该文件会被保留，需要人工改为读取 docs/dev-kit/README.md、docs/dev-kit/ai/AI_OPERATOR_GUIDE.md 和 docs/project/。
- 旧脚本、IDE、CI、Python 虚拟环境是否仍引用旧工具目录。虚拟环境及 node_modules 不建议跨目录复用，应按项目需要重建依赖。
- 旧 runtime-sdk/ 如被复制过来，先保留备份并检查引用；后端开发依赖可从新工具包 project-template/runtime-sdk/ 安装，业务项目不再提交 SDK 源码副本。旧本地副本可加入项目 .gitignore。
- 旧 docs/ 与自有说明先保留。项目自身约定整理到 docs/project/，新的托管文档位于 docs/dev-kit/。

7. 按第 5、6 节处理 Git 规则和构建验证。确认通过后再清理旧工具目录、缓存与重复示例；CLI 不自动删除旧代码。

## 4. 新项目初始化

在工具包之外选择一个新的项目目录：

```sh
asapflow plugin init-project --root <项目目录> --source <工具包目录>/content --offline
cd <项目目录>
asapflow plugin init --root . --code order_check --name "订单校验" --kind full
```

--kind 可使用 backend、frontend 或 full。已存在的插件目录会被拒绝覆盖，不要用 plugin init 更新已有插件。

初始化后的主要目录：

```text
customer-a-plugins/
├── AGENTS.md
├── .gitignore
├── .asapflow/dev-kit.json       # 文档版本、来源与校验记录
├── docs/dev-kit/                # 工具托管的规范、示例和开发指南
├── docs/project/                # 团队维护的项目规则
├── backend/<plugin-code>/
├── frontend/src/pages/<plugin-code>/
└── locales/
```

前端项目首次使用时在 frontend/ 下运行 npm install。后端按 docs/dev-kit/plugin-development/backend-plugin-guide.md 安装依赖。初始化不会安装依赖或配置平台凭据。

## 5. 插件仓库由项目团队管理

### Git 文件规则

首次提交前检查 .gitignore。已有文件不会被 CLI 覆盖，请自行补充至少以下规则：

```gitignore
**/node_modules/
**/.venv/
**/__pycache__/
**/.generated/
**/dist/
**/config.json
**/.env
**/.env.*
```

保留需要交付的配置样板，例如有 .env.sample 时可按实际路径添加例外。不要把工具包二进制、SDK 源码副本、缓存、发布产物或真实令牌提交到项目。

**跨 Windows/macOS/Linux 协作时，在项目自己的 .gitattributes 中添加：**

```gitattributes
docs/dev-kit/** -text
```

该规则让托管文档保持原始字节，避免 Git 自动换行导致 SHA-256 校验不一致。当前 init-project 不自动生成该文件；已有 .gitattributes 只追加规则，不覆盖原有内容。应在第一次提交托管文档前配置。

### 建立或接入项目仓库

项目尚无 .git 时才执行 git init，并配置团队自己的远端。以下命令在项目目录执行：

```sh
git init
git branch -M main
git remote add origin <项目团队自己的仓库地址>
git status --short
```

人工核对待提交文件。提交业务源码、构建配置、依赖锁文件、AGENTS.md、.gitignore、.gitattributes、.asapflow/dev-kit.json 和项目文档。确认无密钥、产物和工具副本后再暂存、提交、推送。

新仓库确认文件无误后，例如：

```sh
git add .gitignore .gitattributes AGENTS.md .asapflow docs backend frontend locales
git diff --cached --name-only
# 再次核对清单后提交：
git commit -m "feat(plugins): 初始化项目插件仓库"
git push -u origin main
```

暂存路径按项目实际存在的文件调整。不要跳过暂存清单检查。

已有仓库保留原分支与远端，不重复执行上述初始化。多人开发按团队既有分支和合并流程协作；这次分离解决工具更新与项目代码混在一起的问题，同一业务文件的并行修改仍需正常合并。

### 文档升级的团队约定

建议由一位项目维护者统一升级 docs/dev-kit/ 和 .asapflow/dev-kit.json，审查后一起提交。其他开发者通过项目 git pull 获取同一版文档，不需要每个人重复 --update。

当前版本记录中，本地 --source 会保存为绝对路径。其他开发者不能依赖同事电脑上的路径；确需检查或升级时，显式传入自己机器的 --source。避免多人只为改本机路径而反复提交版本记录。团队如有统一的 HTTPS 文档内容源，也可使用该地址。

## 6. 日常校验、打包与发布

以下命令在项目根目录运行：

```sh
asapflow plugin validate --root . --code order_check
asapflow plugin pack --root . --code order_check
```

pack 会按需安装并构建前端，制品默认写入项目 dist/。也可用 --output-file 明确指定制品路径。

确认目标平台地址与服务令牌，且明确需要发布后执行：

```sh
asapflow plugin publish --root . --code order_check
```

项目不必携带 plugin-build.* 或 plugin-publish.*。如果使用工具包快捷脚本，应从项目目录调用工具包内脚本，或显式传入项目根目录；脚本不再回退到工具包示例目录。

## 7. 更新项目开发文档

先在工具包目录 git pull --ff-only，获取新的 content/，再进入项目检查：

```sh
asapflow plugin init-project --root . --source <工具包目录>/content --offline --check
```

--check 不写文件。确认需要升级后：

```sh
asapflow plugin init-project --root . --source <工具包目录>/content --offline --update
```

- 文档使用独立内容版本；工具程序未变化时，CLI 版本可以保持不变。
- 更新只管理 docs/dev-kit/ 和文档版本记录，不覆盖 backend/、frontend/、locales/、AGENTS.md 或 docs/project/；前端构建配置也不会自动升级。
- 已有源码需要适配新版 SDK 或构建器时，由项目维护者按发布说明人工审查变更，不整目录覆盖。
- 内容校验失败、版本降级或托管文件被本地修改时，更新会拒绝执行。先备份并判断是实际修改还是换行变化；不要删掉版本记录或强制覆盖来绕过检查。
- 项目特有说明放 docs/project/，不要直接改 docs/dev-kit/。新版移除的旧文档会保留，不自动删除。
- --offline 只使用本地内容源。工具包已在本机时，即使不能联网仍可初始化或更新；获取新的 Git 提交和首次下载依赖仍需要网络。

升级成功后，维护者检查 Git 差异，再将 docs/dev-kit/ 和 .asapflow/dev-kit.json 作为同一次文档升级提交。

## 8. 迁移完成检查

- 工具包与每个项目处于互相独立的目录，没有父子 Git 仓库或子模块依赖。
- 任意项目目录可执行更新后的 asapflow，且未携带 CLI 二进制副本。
- 原业务源码、语言资源及既有项目 .git 均保留。
- 项目读取 docs/dev-kit/ 中的开发规范，自有规则位于 docs/project/。
- Git 忽略规则和托管文档换行规则已配置；没有提交令牌、依赖缓存或发布包。
- 至少一个现有插件完成 validate 和 pack；需要平台联调时另行确认发布环境。
- 工具包更新、项目代码同步和项目文档升级分别操作，彼此不代管 Git。
