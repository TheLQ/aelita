Add new data type

1. Write new SQL migration in `stor_diesel/migrations/`
2. Apply SQL to database
3. Update schema.rs with `stor_diesel/scripts/diesel-schema.sh`
4. Add model to `stor_diesel/src/models`
5. Add read and mut api to `stor_diesel/src/api`
6. Update import if needed in `stor_import`
7. Update frontend if needed in `www`

```raw
Browser History        > Mutation Log > Distilled Database 
Tabs Open/Close Events                  Frontend Site
Reddit Saved
Twitter Saved
Youtube Downloads
Project Tracking
```
