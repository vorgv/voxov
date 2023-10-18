# VOxOV Shell

A federated monolithic pay-as-you-go backend-as-a-service cloud-retail project.

Not ready for production.

Testing server with `$SKIP_AUTH` and `$SAMSARA`
- http://c31.io:8080

Waiting for upstreams to fix security issues from Dependabot.

## Todos

- Integration tests
- Meme deduplication
- Impl gene: geo
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

- api
    - static: large files
    - TODO graphql: nested requests
- auth
    - user
    - TODO fed
        - graphql: reduce trips
    - TODO alien
        - static: reduce inter-cluster traffic
- cost
    - time cost
    - traffic cost
    - space cost
    - tip
- TODO fed
    - optional jwt (for untrusted nodes)
    - exchange rate (static range, local currency)
        - stay stable to avoid financialization
        - changing rate
            - extend range and wait for adaption
            - shrink to complete shift
- gene
    - info
    - map
        - document database
    - msg
        - chat
    - human
    - censor
        - publish
        - argue/report
        - human verification
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
    - blobs (s3)
