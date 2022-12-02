# voxov
Pay-as-you-go BaaS

    go install github.com/deepmap/oapi-codegen/cmd/oapi-codegen@latest
    oapi-codegen -package api -generate types,server,spec api/voxov.yaml > pkg/api/gen.go
