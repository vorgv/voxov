# VOxOV Rain Shell

A federated monolithic pay-as-you-go backend-as-a-service cloud-retail project.

## Todos

- Impl gene: info
- Impl gene: map
- Impl gene: human
- Impl gene: censor
- Impl gene: like
- Impl gene: chan 
- Impl gene: automation
- Impl gene: xr
- Impl gene: wiki
- Impl gene: ai
- Impl fed with jwt
- GraphQL API
- GUI client (Flutter)

## Layers

- api (stateless)
    - static: large files
    - graphql: nested metadata
- auth (redis)
    - user
        - signup, login/logout, pay
    - fed
        - graphql: reduce trips
    - alien
        - static: reduce inter-cluster traffic
- cost (redis, log to mongodb)
    - time cost
    - traffic cost
    - space cost
    - tips
- fed (static only)
    - optional jwt (for untrusted nodes)
    - exchange rate (static range, local currency)
        - stay stable to avoid financialization
        - changing rate
            - extend range and wait for adaption
            - shrink to complete shift
- gene (static)
    - metadata
    - censor
        - publish: at least 6 months remaining
        - argue/report: do expensive publish
        - human verification
    - channels (buffer/cursor)
        - new/delete
        - push
        - pull
        - application: notification, chat, forum
    - map
        - like a filesystem
    - like
        - pay to get
        - spend credit to give
        - leaderboard
    - automation
        - file keep-alive
    - geological XR tools
        - ads
        - social
    - wiki
        - 1:1 server-author likes
    - ai
        - models
    - recommandation system
        - index all public memes
- meme
    - metadata (mongodb)
    - static data (s3)
- database
    - redis
        - set, get, expire
    - mongodb
        - meme metadata
    - S3
        - meme data
