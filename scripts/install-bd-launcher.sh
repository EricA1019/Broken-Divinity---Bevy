#!/usr/bin/env bash
# Install or verify the managed `bd` launcher in the user's zsh configuration.
set -euo pipefail

readonly begin_marker="# >>> Broken Divinity launcher (managed by install-bd-launcher.sh) >>>"
readonly end_marker="# <<< Broken Divinity launcher (managed by install-bd-launcher.sh) <<<"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
launcher="${script_dir}/bd"
zsh_config_dir="${ZDOTDIR:-${HOME}}"
zsh_config="${zsh_config_dir}/.zshrc"
mode="${1:-install}"
temp_config=""

printf -v quoted_launcher '%q' "${launcher}"
managed_block="$(
    printf '%s\n' "${begin_marker}"
    printf 'bd() {\n'
    printf '    %s "$@"\n' "${quoted_launcher}"
    printf '}\n'
    printf '%s\n' "${end_marker}"
)"

cleanup() {
    if [[ -n "${temp_config}" ]] && [[ -f "${temp_config}" ]]; then
        rm -f -- "${temp_config}"
    fi
}

check_installation() {
    if [[ ! -f "${zsh_config}" ]]; then
        printf 'Missing zsh configuration: %s\n' "${zsh_config}" >&2
        return 1
    fi

    local installed_block
    installed_block="$(
        awk \
            -v begin="${begin_marker}" \
            -v end="${end_marker}" \
            '
                $0 == begin { capture = 1 }
                capture { print }
                capture && $0 == end { capture = 0 }
            ' "${zsh_config}"
    )"

    if [[ "${installed_block}" != "${managed_block}" ]]; then
        printf 'The bd launcher is missing or stale in %s\n' "${zsh_config}" >&2
        return 1
    fi

    printf 'The bd launcher is current: %s\n' "${launcher}"
}

install_launcher() {
    mkdir -p "${zsh_config_dir}"
    touch "${zsh_config}"

    temp_config="$(mktemp "${TMPDIR:-/tmp}/bd-zshrc.XXXXXX")"
    trap cleanup EXIT

    if ! awk \
        -v begin="${begin_marker}" \
        -v end="${end_marker}" \
        '
            $0 == begin {
                pending_blank_lines = 0
                in_managed_block = 1
                next
            }
            in_managed_block && $0 == end { in_managed_block = 0; next }
            in_managed_block { next }
            in_legacy_function {
                legacy_line = $0
                legacy_depth += gsub(/{/, "{", legacy_line)
                legacy_depth -= gsub(/}/, "}", legacy_line)
                if (legacy_depth <= 0) {
                    in_legacy_function = 0
                }
                next
            }
            $0 ~ /^[[:space:]]*bd[[:space:]]*\(\)[[:space:]]*\{/ {
                legacy_line = $0
                legacy_depth = gsub(/{/, "{", legacy_line)
                legacy_depth -= gsub(/}/, "}", legacy_line)
                pending_blank_lines = 0
                if (legacy_depth > 0) {
                    in_legacy_function = 1
                }
                next
            }
            $0 == "" {
                pending_blank_lines += 1
                next
            }
            {
                while (pending_blank_lines > 0) {
                    print ""
                    pending_blank_lines -= 1
                }
                print
            }
            END {
                if (in_managed_block || in_legacy_function) {
                    exit 2
                }
            }
        ' "${zsh_config}" >"${temp_config}"; then
        printf 'Refusing to edit %s: its managed bd block is malformed.\n' "${zsh_config}" >&2
        return 1
    fi

    if [[ -s "${temp_config}" ]]; then
        printf '\n' >>"${temp_config}"
    fi
    printf '%s\n' "${managed_block}" >>"${temp_config}"

    chmod --reference="${zsh_config}" "${temp_config}"
    if cmp -s "${temp_config}" "${zsh_config}"; then
        printf 'The bd launcher is already current: %s\n' "${launcher}"
        return 0
    fi

    cp -p "${zsh_config}" "${zsh_config}.bd-backup"
    mv "${temp_config}" "${zsh_config}"
    temp_config=""
    trap - EXIT
    printf 'Installed the bd launcher in %s\n' "${zsh_config}"
    printf 'Backup written to %s\n' "${zsh_config}.bd-backup"
}

case "${mode}" in
    install)
        install_launcher
        ;;
    --check)
        check_installation
        ;;
    *)
        printf 'Usage: %s [--check]\n' "$0" >&2
        exit 2
        ;;
esac
