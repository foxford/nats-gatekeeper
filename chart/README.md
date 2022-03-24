# Nats gatekeeper helm chart

Helm chart for [nats-gatekeeper](https://github.com/foxford/nats-gatekeeper/)

## Prerequisites

`nats-credentials` secret must be present with nats key.

`.der` keys for each audience should exist to authenticate tokens
(see `.audiences.*.authn.key` and `.container.{volumes,volumeMounts}` in `values.yaml`)

## Installation

To install gatekeeper cd into this dir and run
```
helm install nats-gatekeeper . --atomic -n testing01
```

## Tests

You can check that installation completed (somewhat) successfully with
```
helm test nats-gatekeeper -n testing01
```

## Removal

To get rid of this chart run
```
helm uninstall nats-gatekeeper -n helm-test-shkh
```
