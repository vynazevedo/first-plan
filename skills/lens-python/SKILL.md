---
name: first-plan-lens-python
description: Stack lens para Python. Use durante Discovery quando pyproject.toml, setup.py ou requirements.txt for detectado. Cobre FastAPI, Django, Flask, Litestar, Celery workers, data apps, packaging src/ vs flat.
version: 0.1.0
---

# Lens Python

## Detecção fina

| Sinal | Variante |
|-------|----------|
| `manage.py` na raiz | Django |
| `fastapi` em deps | FastAPI |
| `flask` em deps | Flask |
| `litestar` em deps | Litestar |
| `celery` em deps | Worker |
| `streamlit` em deps | Data app |
| `dagster`, `airflow`, `prefect` | Data pipeline |
| `setup.py` apenas | lib legacy |
| `pyproject.toml` com `[tool.poetry]` | Poetry |
| `pyproject.toml` com `[tool.uv]` ou `uv.lock` | uv |
| `pyproject.toml` com `[build-system]` setuptools | setuptools moderno |
| `Pipfile` | pipenv |

## Extração de padrões

### Packaging

- Layout `src/` vs flat
- Namespace package vs regular
- Versão Python suportada (`requires-python`)
- Optional dependencies / extras

### Type checking

- mypy / pyright / pyre / ty
- Configuração strictness (`strict = true`?)
- Cobertura de tipos visível

### Validação

- Pydantic (v1 ou v2?) - dominante em FastAPI
- attrs (validação opcional)
- marshmallow
- dataclasses + manual validation

### Async

- `asyncio` direto
- `anyio` (compatibilidade)
- `trio`
- async libs (httpx vs requests, asyncpg vs psycopg)

### Testing

- pytest (dominante) ou unittest stdlib
- `conftest.py` em pasta tests/ -> fixtures globais
- Markers customizados (`pytest.mark.integration`)
- Coverage tool (coverage.py)
- Mocks: `unittest.mock`, `pytest-mock`, `responses` (HTTP)

### Web frameworks - padrões

FastAPI:
- Routers em `app/routers/`
- Dependency injection via `Depends()`
- Pydantic schemas em `app/schemas/`
- ORM tipicamente SQLAlchemy + Alembic migrations

Django:
- Apps em `app/<name>/` com `models.py`, `views.py`, `urls.py`, `admin.py`
- Settings em `<project>/settings.py`
- Migrations geradas
- Templates em `templates/`

Flask:
- Application factory pattern (`create_app()`)?
- Blueprints
- Extensions (Flask-SQLAlchemy, Flask-Login, etc)

### ORM

- SQLAlchemy 1.x vs 2.x (style mudou drasticamente)
- Django ORM
- Tortoise ORM (async)
- Peewee
- Raw SQL via psycopg / sqlite3

### Logging

- stdlib `logging` (dominante)
- structlog (estruturado)
- loguru (alternativa moderna)

### Linters / formatters

- black + ruff (configuração comum)
- ruff replacing flake8/isort
- mypy / pyright

### Workers / async tasks

- Celery (broker Redis ou RabbitMQ)
- Dramatiq
- arq (asyncio-based)
- RQ

## Output em `.first-plan/`

Padrão. Adicional:
- `01-topology/boundaries.md` - rotas FastAPI/Django/Flask, tasks Celery
- `02-conventions/di.md` - Depends() vs constructor vs factory

## Confidence rules

Aumentar:
- `[tool.mypy] strict = true` + arquivos sem `# type: ignore` -> alta confiança em "tipagem estrita"
- `pyproject.toml` bem definido com tools configuradas

Reduzir:
- Mistura de Python 2 e 3 styles (e.g., `super(Cls, self).__init__()` ao lado de `super().__init__()`)
- Sem type hints em código novo

## Anti-padrões comuns

- `from x import *` (smell)
- `except:` ou `except Exception:` sem logar nem re-raise
- Imports circulares mascarados com import dentro de função
- Mutable default args (`def f(x=[]):`)
- Mistura sync e async sem critério
- `assert` em produção (removido com `python -O`)
