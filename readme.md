# Zero2Prod

Axum + Sea-Orm

# 部署到 Digital Ocean

```bash
# 生成 token
# https://cloud.digitalocean.com/account/api/tokens
doctl auth init --context tonken_name
doctl apps create --spec spec.yaml --context tonken_name
```
