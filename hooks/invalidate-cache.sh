#!/usr/bin/env bash
#
# first-plan invalidate-cache hook
#
# Disparado em PostToolUse de Edit/Write/MultiEdit.
# Lê path do arquivo modificado via stdin (JSON do Claude Code) e:
# 1. Verifica se há .first-plan/ no projeto-alvo (CLAUDE_PROJECT_DIR ou cwd)
# 2. Mapeia path modificado para seções afetadas
# 3. Adiciona path ao set "stale_paths" em .first-plan/cache.json
# 4. Anexa entrada em .first-plan/08-meta/coverage.md seção Stale
#
# NUNCA regenera nada. Apenas sinaliza.
# Falhas silenciosas - nao quebrar fluxo do Claude Code.

set -uo pipefail

# Falha silenciosa - log em arquivo se quiser debug
LOG_FILE="${HOME}/.first-plan-hook.log"

log() {
    echo "[$(date -Iseconds)] $*" >> "${LOG_FILE}" 2>/dev/null || true
}

main() {
    # Lê JSON do stdin (formato Claude Code PostToolUse)
    local input
    input="$(cat 2>/dev/null || true)"
    if [[ -z "${input}" ]]; then
        log "no input - exiting"
        exit 0
    fi

    # Determinar diretório do projeto
    local project_dir="${CLAUDE_PROJECT_DIR:-${PWD}}"

    # Verificar se há .first-plan/ no projeto
    if [[ ! -d "${project_dir}/.first-plan" ]]; then
        log "no .first-plan/ in ${project_dir} - exiting silently"
        exit 0
    fi

    # Extrair path do arquivo modificado do JSON
    # Tenta formatos comuns: tool_input.file_path, tool_input.path
    local file_path
    file_path="$(echo "${input}" | grep -oE '"file_path"[[:space:]]*:[[:space:]]*"[^"]+"' | head -1 | sed -E 's/.*"file_path"[[:space:]]*:[[:space:]]*"([^"]+)"/\1/')"

    if [[ -z "${file_path}" ]]; then
        file_path="$(echo "${input}" | grep -oE '"path"[[:space:]]*:[[:space:]]*"[^"]+"' | head -1 | sed -E 's/.*"path"[[:space:]]*:[[:space:]]*"([^"]+)"/\1/')"
    fi

    if [[ -z "${file_path}" ]]; then
        log "no file_path extracted from input"
        exit 0
    fi

    # Ignorar mudanças no proprio .first-plan/ (não auto-invalidar)
    case "${file_path}" in
        *"/.first-plan/"*|".first-plan/"*)
            log "ignoring change inside .first-plan/: ${file_path}"
            exit 0
            ;;
    esac

    # Path relativo ao project_dir
    local rel_path="${file_path#${project_dir}/}"

    # Determinar seções afetadas baseadas no tipo de arquivo
    local affected_sections=()

    case "${rel_path}" in
        # Manifestos -> 01-topology/stacks
        go.mod|*/go.mod|package.json|*/package.json|composer.json|*/composer.json|pyproject.toml|*/pyproject.toml|Cargo.toml|*/Cargo.toml|Gemfile|*/Gemfile|pubspec.yaml|*/pubspec.yaml|*.csproj|*/*.csproj)
            affected_sections+=("01-topology/stacks")
            ;;
    esac

    case "${rel_path}" in
        # CI/CD / Docker -> 01-topology/deployments
        Dockerfile|*/Dockerfile|docker-compose.*|.github/workflows/*|.gitlab-ci.yml|.circleci/config.yml|Makefile|*/Makefile)
            affected_sections+=("01-topology/deployments")
            ;;
    esac

    case "${rel_path}" in
        # IaC -> 01-topology/stacks + deployments
        *.tf|*.tofu)
            affected_sections+=("01-topology/stacks" "01-topology/deployments")
            ;;
    esac

    case "${rel_path}" in
        # Docs/specs -> 09-features
        docs/*|specs/*|requirements/*|rfcs/*|README.md|CHANGELOG.md|*.md)
            affected_sections+=("09-features")
            ;;
    esac

    case "${rel_path}" in
        # Tests -> 02-conventions/testing + 05-risks/untested
        *_test.go|*test_*.py|*.test.ts|*.test.tsx|*.spec.ts|*.spec.tsx|*.test.js|*Test.php|*Tests.php|tests/*|test/*|spec/*|__tests__/*)
            affected_sections+=("02-conventions/testing" "05-risks/untested")
            ;;
    esac

    case "${rel_path}" in
        # Source code geral -> activity + conventions
        src/*|cmd/*|internal/*|pkg/*|lib/*|app/*|*.go|*.ts|*.tsx|*.js|*.py|*.php|*.rs|*.java|*.kt|*.swift)
            affected_sections+=("01-topology/activity" "02-conventions/naming" "02-conventions/errors" "02-conventions/logging")
            ;;
    esac

    # Reuse paths típicos
    case "${rel_path}" in
        pkg/*|lib/*|utils/*|helpers/*|shared/*|*/pkg/*|*/lib/*|*/utils/*|*/helpers/*|*/shared/*)
            affected_sections+=("03-reuse")
            ;;
    esac

    # Components / hooks (frontend)
    case "${rel_path}" in
        */components/*|components/*|*/hooks/*|hooks/*)
            affected_sections+=("03-reuse")
            ;;
    esac

    # Domain / models
    case "${rel_path}" in
        */models/*|models/*|*/entities/*|entities/*|*/domain/*|domain/*)
            affected_sections+=("04-domain/entities")
            ;;
    esac

    # Sem seção afetada -> sair
    if [[ ${#affected_sections[@]} -eq 0 ]]; then
        log "no affected sections for ${rel_path}"
        exit 0
    fi

    # Deduplicar
    local unique_sections
    unique_sections=$(printf '%s\n' "${affected_sections[@]}" | sort -u)

    log "stale: ${rel_path} -> $(echo ${unique_sections} | tr '\n' ' ')"

    # Append em coverage.md (apenas string append - simples e robusto)
    local coverage_file="${project_dir}/.first-plan/08-meta/coverage.md"
    local timestamp
    timestamp="$(date -Iseconds)"

    if [[ -f "${coverage_file}" ]]; then
        # Verifica se ja tem o path listado para evitar duplicidade
        if ! grep -qF "${rel_path}" "${coverage_file}" 2>/dev/null; then
            # Adiciona em uma seção especial no final do arquivo
            {
                echo ""
                echo "<!-- stale-entry timestamp=${timestamp} -->"
                echo "- \`${rel_path}\` (modificado em ${timestamp}) - afeta: $(echo ${unique_sections} | tr '\n' ',')"
            } >> "${coverage_file}" 2>/dev/null || true
        fi
    fi

    # Atualizar cache.json adicionando ao stale_paths array
    # Implementação simples - sem depender de jq que pode não existir
    local cache_file="${project_dir}/.first-plan/08-meta/cache.json"
    if [[ -f "${cache_file}" ]]; then
        # Marcador simples - registrar staleness em arquivo .stale separado
        # (mais robusto que mexer em JSON sem jq)
        local stale_marker="${project_dir}/.first-plan/cache/.stale"
        mkdir -p "${project_dir}/.first-plan/cache" 2>/dev/null
        if [[ ! -f "${stale_marker}" ]] || ! grep -qF "${rel_path}" "${stale_marker}" 2>/dev/null; then
            echo "${rel_path}" >> "${stale_marker}" 2>/dev/null || true
        fi
    fi

    exit 0
}

main "$@"
