# 命令示例

## 查看帮助

```bash
./legacy-export --help
./legacy-export export-scope --help
```

## 单功能探查

```bash
./legacy-export inspect-function --config config.json --func-id SD_T_SO
```

## 单功能导出

```bash
./legacy-export export-function --config config.json --func-id SD_T_SO --out exports
```

## 按菜单目录导出

```bash
./legacy-export export-menu-subtree --config config.json --menu-name "站点配送" --out exports
```

## 按范围文件导出

```bash
./legacy-export export-scope --config config.json --scope-file examples/scope.json --out exports
```

## 批量功能导出

```bash
./legacy-export batch-export --config config.json --func-file examples/funcs.txt --out exports
```

## 全系统配置与系统数据导出

```bash
./legacy-export export-system --config config.json --out exports
```

## 分页查询业务数据

```bash
./legacy-export query-data \
  --config config.json \
  --table BD_ITEM \
  --columns CODE_ITEM,DESC_ITEM,TYPE_ITEM,STAT_ITEM \
  --where "STAT_ITEM='1'" \
  --order-by "CODE_ITEM ASC" \
  --pageIndex 1 \
  --pageSize 1000 \
  --format json \
  --out exports/business_data/BD_ITEM/page-1.json
```
