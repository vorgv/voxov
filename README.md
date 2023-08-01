# VOxOV Rain Shell

A federated monolithic pay-as-you-go backend-as-a-service cloud-retail project.

Not ready for production. I patch dependencies without forking. This crate may not compile on your machine.

## Todos

- Impl gene: map
- Impl gene: chan
- Impl gene: notify
- Impl gene: human
- Impl gene: censor
- Impl gene: like
- Impl gene: auto
- Impl gene: xr
- Impl gene: wiki
- Impl gene: ai
- Impl fed with jwt
- GraphQL API
- GUI client (Flutter? Svelte?)

## Layers

- api (stateless)
    - static: large files
    - TODO graphql: nested metadata
- auth (redis)
    - user
        - signup, login/logout, pay
    - TODO fed
        - graphql: reduce trips
    - TODO alien
        - static: reduce inter-cluster traffic
- cost (redis, TODO log to mongodb)
    - time cost
    - traffic cost
    - space cost
    - tip
- TODO fed (static only)
    - optional jwt (for untrusted nodes)
    - exchange rate (static range, local currency)
        - stay stable to avoid financialization
        - changing rate
            - extend range and wait for adaption
            - shrink to complete shift
- gene (static, mostly TODOs)
    - info
    - map
        - document database
    - human
    - censor
        - publish: at least 6 months remaining
        - argue/report: do expensive publish
        - human verification
    - chan (buffer/cursor)
        - new/delete
        - push
        - pull
        - application: chat, forum
    - notify
    - like
        - pay to get
        - spend credit to give
        - leaderboard
    - auto
        - file keep-alive
    - xr
        - ads
        - social
    - wiki
        - 1:1 server-author likes
    - ai
        - model rpc
    - feed
        - ai powered meme stream
- meme
    - metadata (mongodb)
    - static data (s3)
