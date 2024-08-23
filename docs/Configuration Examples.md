Example of basic cloud setup configuration file:
```toml
[databases]
  [database.main]
  type = "postgres"

[stores]
  [store.uploaded-files]

[lambdas]
  [lambda.setup-users]
  root = "lambdas/setup-users"
  read = [ "database.main", "storage.uploaded-files" ]
  write = [ "database.main", "storage.uploaded-files" ]
  
  [lambda.send-notifications]
  root = "lambdas/send-notifications"
  read = [ "database.main" ]
  write = [ "store.uploaded-files" ]

[hosting]
  [hosting.main]
  root = "static-site/"
```

