# VOxOV - the universal server

![Matrix](https://img.shields.io/matrix/voxov%3Amatrix.org)
![Docker Pulls](https://img.shields.io/docker/pulls/c31io/voxov.svg)
![Docker Image Size](https://img.shields.io/docker/image-size/c31io/voxov.svg)

## Databases

- Fast: CQL, ScyllaDB
- Hash: BLAKE3, S3, CDN
- Sync: SQL, CockroachDB

## Deprecated

I am planning a major design change.
All of the following is deprecated.

A federated monolithic pay-as-you-go backend-as-a-service project.

Testing server is offline.

To setup your own server,

    git clone --depth=1 https://github.com/vorgv/voxov
    cd voxov
    docker compose up

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

## Testing

Start the databases.

    cd ./deploy/docker/databases
    docker compose up

Build and start the server.

    cargo run

Run tests.

    cargo test

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
