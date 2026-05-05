---
name: first-plan-lens-go
description: Stack lens para projetos Go. Use durante Discovery quando go.mod for detectado. Extrai padrões de layout (cmd/internal/pkg), error handling, context.Context, concurrency, code generation, build tags.
version: 0.1.0
---

# Lens Go

## Detecção fina

- `cmd/<name>/main.go` -> binário (CLI ou serviço HTTP)
- `internal/` -> lib privada do módulo
- `pkg/` -> lib pública
- `go.mod` em raiz com poucas pastas -> lib pequena
- Workspace `go.work` -> monorepo Go

Frameworks comuns a detectar via imports:
- `net/http` puro = HTTP stdlib
- `chi`, `gin`, `echo`, `fiber`, `gorilla/mux` = router
- `gqlgen`, `graph-gophers/graphql-go` = GraphQL
- `grpc-go` = gRPC
- `cobra`, `urfave/cli` = CLI
- `kafka-go`, `segmentio/kafka-go`, `IBM/sarama` = Kafka
- `streadway/amqp`, `rabbitmq/amqp091-go` = RabbitMQ

## Extração de padrões

### Layout
- Confirmar uso de `cmd/`, `internal/`, `pkg/`
- Onde fica o entry point principal (`cmd/<name>/main.go`?)
- Há separação `handler/`, `service/`, `repository/`?

### Error handling
- `errors.New` vs `fmt.Errorf` vs `errors.Wrap`
- Sentinel errors (var ErrNotFound = errors.New(...))
- Custom error types (struct com `Error() string`)
- Use de `errors.Is` / `errors.As`
- Pacote externo (`pkg/errors`, `cockroachdb/errors`, `hashicorp/go-multierror`)

Verificar:
- Os handlers retornam erro tipado ou string?
- Errors são wrapped com contexto adicional?
- Há padrão para mapear erro -> HTTP status?

### Context
- `context.Context` é primeiro parametro? (idiomatic)
- `context.Background()` ou `context.TODO()` em produção? (smell)
- Uso de `context.WithTimeout` / `WithCancel`

### Concurrency
- Goroutines + channels?
- `sync.WaitGroup` para coordenar?
- `errgroup.Group` (golang.org/x/sync)?
- Pools de worker?

### Code generation
- `go generate` directive em arquivos?
- `mockgen`, `mockery` para mocks?
- `sqlc`, `oapi-codegen`, `protoc-gen-go`?
- Geradores customizados (sinal: arquivos `*.gen.go`, `*.pb.go`, `wire_gen.go`)

### Build tags
- `//go:build <tag>` em arquivos
- Tags comuns: `integration`, `e2e`, `wireinject`

### Testing
- `_test.go` irmão de cada arquivo?
- `testify/assert` ou stdlib?
- Table-driven (presença de `t.Run` em loop)?
- Testes de integração separados (build tag `integration`)?
- Mocks: gerados ou manuais?

### DI
- Injeção manual via construtores (`NewService(deps...)`)
- `wire` (Google) com `wire.go`/`wire_gen.go`?
- `fx` (Uber)?

### Logging
- `log` stdlib?
- `slog` (Go 1.21+)?
- `logrus`, `zap`, `zerolog`?
- Padrão de campos estruturados?

## Output em `.first-plan/`

| Categoria | Arquivo | Conteúdo |
|-----------|---------|----------|
| Stack | `01-topology/stacks.md` | versão Go, framework, papel |
| Layout | `01-topology/architecture.md` | diagrama cmd/internal/pkg |
| Boundaries | `01-topology/boundaries.md` | rotas HTTP, gRPC services, mensageria |
| Errors | `02-conventions/errors.md` | padrão Go-específico |
| Testing | `02-conventions/testing.md` | testify vs stdlib, table-driven |
| DI | `02-conventions/di.md` | manual / wire / fx |
| Logging | `02-conventions/logging.md` | slog / zap / etc |
| Reuse | `03-reuse/utils.md` | funções em `pkg/`, helpers em `internal/util/` |

## Confidence rules

Aumentar quando:
- Mais de 1 manifesto (vários binários no monorepo) com mesmo padrão
- Padrão usado em arquivos recentes E antigos

Reduzir quando:
- Mistura de `errors.New` e `fmt.Errorf` sem padrão claro
- Mistura de `log` stdlib + `slog` + `zap` (migração em curso)
- Vários `context.TODO()` em produção (debt)

## Anti-padrões comuns

Reportar em `06-rationale/dont.md` se detectado:
- `panic()` em handler HTTP sem recover middleware
- Goroutines sem cancellation via context
- `time.Sleep` em código de produção (em vez de `time.After` + select)
- Erro ignorado com `_, _ = ...`
- `init()` com side effects pesados
