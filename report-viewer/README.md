# Phase1 Report Viewer (React + TS)

用于离线查看 Phase1 工作流角色协作过程，风格参考 Playwright report（时间线 + 详情面板）。

## 启动

```bash
cd report-viewer
npm install
npm run dev
```

默认读取数据目录：

- `/week1/collaboration_log.json`
- `/week1/blackboard_state.json`

也可以在页面点击 `Load Folder`，直接选择本地目录（需包含这两个 JSON 文件）。

## 离线构建

```bash
npm run build
npm run preview
```

构建产物在 `dist/`，可作为静态文件离线打开。

## 切换数据目录

通过 URL 参数 `data` 指定报告目录，例如：

- `http://localhost:4173/?data=/week1`
