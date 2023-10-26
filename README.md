# VOxOV Shell

A federated monolithic pay-as-you-go backend-as-a-service cloud-retail project.

Not ready for production.

Testing server with `$SKIP_AUTH` and `$SAMSARA`
- http://c31.io:8080

Waiting for upstreams to fix security issues from Dependabot.

## Todos

- Integration tests     0.0.1
- Meme deduplication    0.0.2
- Impl gene: geo        0.0.3

- Impl gene: notify     0.1.0
- Impl gene: human      0.1.1
- Impl gene: censor     0.1.2
- Impl gene: like       0.1.3
- Impl gene: wiki       0.2.2

- Impl gene: auto       0.2.0
- Impl gene: rpc        0.2.3
- Impl gene: xr         0.2.1

- Impl fed with jwt     0.3.0

- GraphQL API           1.0.0

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
    - rpc
        - AI model rpc
    - feed
        - ai powered meme stream
- meme
    - metadata (mongodb)
    - blobs (s3)
