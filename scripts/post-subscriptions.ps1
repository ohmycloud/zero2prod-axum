$postParams = @{name='fname';email='abc@gmail.com'}
$header = @{"Content-Type"="application/x-www-form-urlencoded"}
Invoke-WebRequest -Method POST -Body $postParams -Headers $header '127.0.0.1:8000/subscriptions'