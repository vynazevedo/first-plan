---
name: first-plan-lens-php
description: Stack lens para PHP. Use durante Discovery quando composer.json for detectado. Cobre Laravel, Symfony, Slim, Hyperf, Laminas, raw PHP. Identifica PSR compliance, ORM, padrões de fila.
version: 0.1.0
---

# Lens PHP

## Detecção fina

| Sinal | Variante |
|-------|----------|
| `artisan` na raiz | Laravel |
| `bin/console` + `symfony/framework-bundle` em deps | Symfony |
| `slim/slim` em deps | Slim |
| `hyperf/hyperf` em deps | Hyperf |
| `laminas/laminas-mvc` em deps | Laminas (Zend) |
| `index.php` em `public/` sem framework | Legacy / vanilla |
| `wp-config.php` | WordPress |

## Extração de padrões

### Composer

- PSR-4 autoload mappings (qual namespace -> qual pasta)
- Versão min de PHP (`require.php`)
- Dev dependencies relevantes (phpunit, phpstan, psalm, php-cs-fixer, rector)

### Estrutura

Laravel:
- `app/Http/Controllers`, `app/Models`, `app/Services`, `app/Repositories`
- `routes/{web,api,console,channels}.php`
- `database/migrations`, `database/seeders`, `database/factories`
- `config/`, `resources/`, `bootstrap/`

Symfony:
- `src/Controller`, `src/Entity`, `src/Repository`, `src/Service`
- `config/services.yaml`, `config/routes.yaml`
- `migrations/`

### Dependency Injection

- Laravel: container service, providers em `app/Providers/`, binding em `register()`
- Symfony: services.yaml, autowire, autoconfigure
- Manual em vanilla

### Errors

- Exceptions customizadas (extends Exception ou DomainException)
- Handler global (Laravel: `app/Exceptions/Handler.php`; Symfony: ExceptionListener)
- HTTP error mapping

### Validação

- Laravel: Form Requests em `app/Http/Requests/`, rules
- Symfony: Validator constraints / annotations
- Manual via if's

### ORM

- Eloquent (Laravel)
- Doctrine (Symfony)
- Query Builder direto
- PDO raw

### Testing

- PHPUnit (mais comum)
- Pest (alternativa moderna)
- Pasta `tests/` com `Unit`, `Feature`, `Integration`
- Factories Laravel / fixtures Doctrine
- Mocks com Mockery

### Filas / Mensageria

- Laravel: Horizon, jobs em `app/Jobs/`
- Symfony: Messenger
- Hyperf: AMQP, Kafka

### Static analysis

- PHPStan level configurado (8 = strict)
- Psalm
- PHP-CS-Fixer rules
- Rector rules

### PSR compliance

- PSR-12 (style) - verificar via php-cs-fixer config
- PSR-7 (HTTP messages) - check em libs como `slim/psr7`, `nyholm/psr7`
- PSR-15 (HTTP middleware)
- PSR-3 (logger interface)

### Logging

- Laravel: `Log` facade (Monolog underneath)
- Symfony: Monolog
- Configuração de canais

## Output em `.first-plan/`

Igual estrutura padrão. Atenção especial:
- `02-conventions/di.md` - container vs autowire vs manual
- `01-topology/boundaries.md` - rotas, jobs, comandos artisan/console

## Confidence rules

Aumentar:
- Configuração estática (phpstan.neon, .php-cs-fixer.dist.php) presente e respeitada
- PSR autoload bem formado em composer.json

Reduzir:
- Mistura de namespaces antigos (`include` direto) e PSR-4
- Migrations órfãs vs models que existem mas não têm migration

## Anti-padrões comuns

- `include`/`require` em vez de autoload
- `$_GET`, `$_POST` direto em handler
- SQL concatenado (em vez de prepared statement / ORM)
- Lógica em controller (Laravel - sinal de service ausente)
- God models (Eloquent model com 50+ métodos)
