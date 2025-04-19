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

# 启动 Redis

```bash
. ./scripts/init-redis.sh
cargo test redirect_to_admin_dashboard_after_login_success
```

测试时需要启动 Redis 数据库, 否则程序无法构建:

```
Failed to build application.: IO Error: Os { code: 61, kind: ConnectionRefused, message: "Connection refused" }
```
