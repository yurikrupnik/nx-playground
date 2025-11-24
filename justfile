#!/usr/bin/env just --justfile

default:
  just -l

_docker-up:
  docker compose -f manifests/dockers/compose.yaml up -d
# Remove local env db
docker-down:
  docker compose -f manifests/dockers/compose.yaml down

run *args:
  bacon {{args}}

_migration:
  sqlx migrate run --database-url=postgres://myuser:mypassword@localhost/mydatabase --source manifests/migrations/postgres/

_seed:
  sqlx migrate run --ignore-missing --database-url=postgres://myuser:mypassword@localhost/mydatabase --source manifests/migrations/postgres/seeds

sort-deps:
  cargo fmt
  cargo sort --workspace

reset-db:
  just docker-down
  docker volume prune -af
  just _docker-up
  just _migration
  just _seed

schema:
  cargo run --bin schema-gen -- --format all -o docs
#  docker rm $(docker ps -aq) -f
test-all:
  cargo nextest run --workspace
buf:
    buf lint
    buf build
    buf generate

backstage-dev:
  kubectl apply -k manifests/kustomize/backstage/overlays/dev

backstage-prod:
  kubectl apply -k manifests/kustomize/backstage/overlays/prod

backstage-logs:
  kubectl logs -n backstage deployment/backstage -f

backstage-catalog-generate:
  nu scripts/nu/generate-backstage-catalog.nu

crossplane-functions-install:
  echo 'apiVersion: pkg.crossplane.io/v1beta1\nkind: Function\nmetadata:\n  name: function-kcl\nspec:\n  package: docker.io/kcllang/function-kcl:latest' | kubectl apply -f -
  echo 'apiVersion: pkg.crossplane.io/v1beta1\nkind: Function\nmetadata:\n  name: function-cue\nspec:\n  package: docker.io/crossplane-contrib/function-cue:latest' | kubectl apply -f -

backstage-setup-github:
  nu scripts/nu/backstage-setup-providers.nu github

backstage-setup-aws:
  nu scripts/nu/backstage-setup-providers.nu aws

backstage-setup-gcp:
  nu scripts/nu/backstage-setup-providers.nu gcp

backstage-setup-cloudflare:
  nu scripts/nu/backstage-setup-providers.nu cloudflare

backstage-setup-all:
  nu scripts/nu/backstage-setup-providers.nu all

backstage-restart:
  kubectl rollout restart deployment/backstage -n backstage
  kubectl rollout status deployment/backstage -n backstage

dsa:
  nx add @nx/vite
  nx g @nx/vite:app web --directory=apps/zerg/web --unitTestRunner=vitest --projectNameAndRootFormat=as-provided
  cargo run -- migrate up # test is why it is up like this
