# Zero2Prod

Axum + Sea-Orm

# 部署到 Digital Ocean

```bash
# 生成 token
# https://cloud.digitalocean.com/account/api/tokens
doctl auth init --context tonken_name
doctl apps create --spec spec.yaml --context tonken_name
```

# 查询数据库

```bash
# 连接 postgres 数据库
psql -h localhost -p 5432 -U postgres

# 列出数据库
\l newsletter
\c newsletter
\dt

newsletter=# \dt
                List of relations
 Schema |        Name         | Type  |  Owner
--------+---------------------+-------+----------
 public | _sqlx_migrations    | table | postgres
 public | seaql_migrations    | table | postgres
 public | subscription_tokens | table | postgres
 public | subscriptions       | table | postgres


delete from subscriptions;
```
