#!/usr/bin/env bash

# Extract RHEL entitlement certificates by registering a container
# These can then be mounted into other containers for subscription content access
#
# Use this when the Konflux activation key is network-restricted and you need
# to regenerate RPM lockfiles locally.

set -euo pipefail

# Configuration
default_entitlement_dir="$HOME/.rhel-entitlements"
default_rh_user="${USER}@redhat.com"

print_status() {
    echo "[INFO] $1"
}

print_success() {
    echo "[SUCCESS] $1"
}

print_warning() {
    echo "[WARNING] $1"
}

print_error() {
    echo "[ERROR] $1"
}

usage() {
    cat << EOF
Usage: ${0##*/} [OPTIONS]

Extract RHEL entitlement certificates by registering a UBI container.
The entitlements can then be used with generate-rpm-lockfile-rhel.sh or
mounted directly into containers that need subscription content access.

OPTIONS:
    -o, --output DIR    Output directory (default: ${default_entitlement_dir})
    -u, --user USER     Red Hat username (default: ${default_rh_user})
    -h, --help          Show this help message

ENVIRONMENT VARIABLES:
    RH_USER             Red Hat username (alternative to --user)
    RH_PASS             Red Hat password (required, or uses 'pass rhat/access.redhat.com')

EXAMPLES:
    # Using pass for password
    ${0##*/}

    # Using environment variable for password
    RH_PASS=\$(cat /path/to/password) ${0##*/}

    # Custom output directory
    ${0##*/} -o /tmp/entitlements

USAGE WITH LOCKFILE GENERATION:
    After extracting entitlements, you can regenerate the RPM lockfile:

    1. Update redhat.repo with your cert ID:
       CERT_ID=\$(ls ~/.rhel-entitlements/*.pem | grep -v key | sed 's/.*\\/\\([0-9]*\\)\\.pem/\\1/')
       OLD_ID=\$(grep -oP 'entitlement/\\K[0-9]+' redhat.repo | head -1)
       sed -i "s/\$OLD_ID/\$CERT_ID/g" redhat.repo

    2. Run the lockfile generator:
       podman run --rm \\
           -v "\$(pwd):/work:Z" \\
           -v "\$HOME/.rhel-entitlements:/etc/pki/entitlement:Z" \\
           -w /work \\
           -e "RPM_LOCKFILE_VERSION=v0.13.1" \\
           registry.access.redhat.com/ubi9 bash -c '
       dnf install -y pip skopeo perl-interpreter
       pip install --quiet --user \\
           "https://github.com/konflux-ci/rpm-lockfile-prototype/archive/refs/tags/\${RPM_LOCKFILE_VERSION}.tar.gz"
       ~/.local/bin/rpm-lockfile-prototype \\
           --image registry.access.redhat.com/ubi9/ubi-minimal:latest \\
           --outfile rpms.lock.yaml \\
           rpms.in.yaml
       '

    3. Revert redhat.repo before committing:
       git checkout redhat.repo

EOF
}

check_requirements() {
    print_status "Checking requirements..."

    if ! command -v podman &> /dev/null; then
        print_error "podman is required but not installed"
        exit 1
    fi

    print_success "Requirements check passed"
}

get_password() {
    # If RH_PASS is already set, use it
    if [[ -n "${RH_PASS:-}" ]]; then
        return 0
    fi

    # Try to get password from pass
    if command -v pass &> /dev/null; then
        if pass show rhat/access.redhat.com &>/dev/null; then
            RH_PASS=$(pass rhat/access.redhat.com | head -1)
            if [[ -n "$RH_PASS" ]]; then
                print_status "Retrieved password from pass store"
                return 0
            fi
        fi
    fi

    print_error "RH_PASS environment variable is required"
    print_error "Set it directly or ensure 'pass rhat/access.redhat.com' is available"
    exit 1
}

extract_entitlements() {
    local output_dir="$1"
    local rh_user="$2"

    print_status "Creating output directory: $output_dir"
    mkdir -p "$output_dir"

    print_status "Registering container and extracting entitlements..."
    print_status "Username: $rh_user"

    if ! podman run --rm \
         -e "RH_USER=${rh_user}" \
         -e "RH_PASS=${RH_PASS}" \
         -v "${output_dir}:/output:Z" \
         registry.access.redhat.com/ubi9 bash -c '
set -e
subscription-manager register --username="$RH_USER" --password="$RH_PASS" >/dev/null
echo "Registered successfully"
cp /etc/pki/entitlement/*.pem /output/
echo "Entitlements copied"
subscription-manager unregister >/dev/null 2>&1 || true
'; then
        print_error "Failed to extract entitlements"
        exit 1
    fi

    if ls "${output_dir}"/*.pem &>/dev/null; then
        print_success "Entitlements extracted to: $output_dir"
        ls -la "$output_dir"/*.pem

        local cert_id
        cert_id=$(ls "${output_dir}"/*.pem | grep -v key | head -1 | sed 's/.*\/\([0-9]*\)\.pem/\1/')
        echo ""
        print_status "Certificate ID: $cert_id"
        print_status "To use with podman:"
        echo "  podman run -v ${output_dir}:/etc/pki/entitlement:Z ..."
    else
        print_error "No entitlement certificates found"
        exit 1
    fi
}

# Parse command line arguments
entitlement_dir="$default_entitlement_dir"
rh_user="${RH_USER:-$default_rh_user}"

while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            entitlement_dir="$2"
            shift 2
            ;;
        -u|--user)
            rh_user="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

print_status "Starting RHEL entitlement extraction..."

check_requirements
get_password
extract_entitlements "$entitlement_dir" "$rh_user"

print_success "Entitlement extraction completed!"
