# 命令示例

## 查看帮助

```bash
./target/release/legacy-export --help
./target/release/legacy-export export-scope --help
```

## 单功能探查

```bash
./target/release/legacy-export inspect-function --config config.json --func-id SD_T_SO
```

## 单功能导出

```bash
./target/release/legacy-export export-function --config config.json --func-id SD_T_SO --out exports
```

## 按菜单目录导出

```bash
./target/release/legacy-export export-menu-subtree --config config.json --menu-name "站点配送" --out exports
```

## 按范围文件导出

```bash
./target/release/legacy-export export-scope --config config.json --scope-file examples/scope.json --out exports
```

## 批量功能导出

```bash
./target/release/legacy-export batch-export --config config.json --func-file funcs.txt --out exports
```

## 全系统配置与系统数据导出

```bash
./target/release/legacy-export export-system --config config.json --out exports
```
