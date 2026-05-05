---
name: first-plan-lens-terraform
description: Stack lens para Terraform e IaC similar (OpenTofu, Pulumi). Use durante Discovery quando arquivos *.tf forem detectados. Cobre estrutura de módulos, backend de state, environments, providers, naming/tagging.
version: 0.1.0
---

# Lens Terraform / IaC

## Detecção

- `*.tf` em qualquer pasta
- `*.tofu` (OpenTofu)
- `Pulumi.yaml` (Pulumi - alternativa, mas não Terraform)
- `cdk.json` (AWS CDK)
- `serverless.yml` (Serverless Framework)

## Estrutura

### Layout comum

```
infra/
├── modules/             módulos reutilizáveis
│   ├── network/
│   ├── compute/
│   └── database/
├── environments/        ou `live/`
│   ├── dev/
│   ├── staging/
│   └── prod/
└── shared/              providers, vars compartilhadas
```

Variantes:
- Workspaces (`terraform workspace`) - menos comum em produção
- Pasta única com tfvars por env

### Detectar

- Onde está o root módulo (entry para `terraform apply`)?
- Há módulos em `modules/`?
- Há separação por env (`environments/<env>/main.tf`)?
- Quais providers usados? (aws, azure, gcp, kubernetes, helm, etc)

## Backend de state

- `backend "s3"` + DynamoDB lock?
- `backend "azurerm"` ou `backend "gcs"`?
- `backend "remote"` (Terraform Cloud)?
- Local state (`backend "local"`) - red flag para prod

## Versionamento

- `terraform.required_version` pinned?
- `required_providers` com versão pinned?
- `.terraform-version` (tfenv) ou `.tool-versions` (asdf)?

## Convenções

### Naming

- Recursos: `<projeto>-<ambiente>-<componente>`?
- Tags obrigatórias: Environment, Project, Owner, CostCenter?
- Module naming convention

### Variables

- `variables.tf` + `outputs.tf` em cada módulo
- `terraform.tfvars` ou `*.auto.tfvars`?
- Validation blocks?
- Sensitive marking?

### Outputs

- Outputs nomeados consistentemente
- Modulos consumidos via `module.X.output_name`?
- Outputs sensíveis marcados?

## Padrões de segurança

- Hardcoded secrets? (red flag)
- Use de `aws_secretsmanager`, `vault`, `sops`
- IAM least privilege?
- Encryption at rest configurada nos resources

## CI/CD

- `terraform plan` em PR?
- Atlantis, terraform-cloud, scalr, env0?
- GitHub Actions / GitLab CI / etc.

## Testing

- `terratest` (Go-based)?
- `kitchen-terraform`?
- `tflint`, `tfsec`, `checkov` em CI?

## Output em `.first-plan/`

| Categoria | Conteúdo |
|-----------|----------|
| `01-topology/stacks.md` | Terraform version, providers, modules |
| `01-topology/architecture.md` | Mapa de módulos e dependências |
| `01-topology/deployments.md` | Backend de state, CI/CD, environments |
| `02-conventions/naming.md` | Convenção de naming de recursos + tagging |
| `02-conventions/security.md` | IAM patterns, secret management |
| `03-reuse/components.md` | Módulos reutilizáveis (cada um vira entry) |

## Confidence rules

Aumentar:
- Tagging consistente em todos os recursos
- Versões pinned (provider + terraform)
- Backend remoto com locking

Reduzir:
- Mistura de styles (alguns recursos com tags, outros sem)
- `count` e `for_each` misturados sem critério
- Local state em pastas de prod

## Anti-padrões comuns

- Hardcoded secrets (acesskey, password) - bloquear
- `terraform.tfstate` commitado (gitignore esquecido)
- Resources sem tags
- Modules sem `variables.tf` ou `outputs.tf` claros
- `null_resource` com `local-exec` - smell, considerar approach declarativo
