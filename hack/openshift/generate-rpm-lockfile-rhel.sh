#!/usr/bin/env bash

# Script to generate RPM lockfile using RHEL repos with activation key
# Use this when packages require subscription (e.g., libbpf-devel from CRB)

set -euo pipefail

# Configuration
rpms_in_file="rpms.in.yaml"
rpms_lock_file="rpms.lock.yaml"
redhat_repo_file="redhat.repo"
default_base_image="registry.access.redhat.com/ubi9/ubi-minimal:latest"
rpm_lockfile_version="v0.13.1"

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

Generate RPM lockfile using RHEL repos with Red Hat activation key.
Use this when packages require subscription content (e.g., CodeReady Builder).

OPTIONS:
    -k, --activation-key KEY  Activation key name (required)
    -O, --org ORG             Red Hat organisation ID (required)
    -i, --input FILE          Input rpms.in.yaml file (default: ${rpms_in_file})
    -o, --output FILE         Output rpms.lock.yaml file (default: ${rpms_lock_file})
    -r, --repo-file FILE      Output redhat.repo file (default: ${redhat_repo_file})
    -b, --base-image IMAGE    Base container image (default: ${default_base_image})
    --list-repos              List repos enabled by the activation key and exit
    -h, --help                Show this help message

ENVIRONMENT VARIABLES:
    RH_ACTIVATION_KEY         Alternative to --activation-key
    RH_ORG_ID                 Alternative to --org

EXAMPLES:
    ${0##*/} -k my-activation-key -O 12345678
    ${0##*/} --activation-key my-key --org 12345678 -b registry.access.redhat.com/ubi9

REQUIREMENTS:
    - podman must be installed and available
    - ${rpms_in_file} must exist in current directory
    - Valid Red Hat activation key with required repos enabled
    - Internet connection for accessing Red Hat CDN

ACTIVATION KEY SETUP:
    1. Create activation key at https://console.redhat.com/insights/connector/activation-keys
    2. Add required repositories (e.g., CodeReady Linux Builder for RHEL 9)
    3. Note your organisation ID (visible in portal URL or account settings)

EOF
}

check_requirements() {
    print_status "Checking requirements..."

    if ! command -v podman &> /dev/null; then
        print_error "podman is required but not installed"
        exit 1
    fi

    if [[ ! -f "$rpms_in_file" ]]; then
        print_error "Input file $rpms_in_file not found"
        exit 1
    fi

    if [[ -z "${activation_key:-}" ]]; then
        print_error "Activation key is required. Use -k/--activation-key or set RH_ACTIVATION_KEY"
        exit 1
    fi

    if [[ -z "${org_id:-}" ]]; then
        print_error "Organisation ID is required. Use -O/--org or set RH_ORG_ID"
        exit 1
    fi

    print_success "Requirements check passed"
}

list_repos() {
    print_status "Listing repos enabled by activation key..."

    podman run --rm \
         -e "RH_ORG_ID=${org_id}" \
         -e "RH_ACTIVATION_KEY=${activation_key}" \
         registry.access.redhat.com/ubi9 \
         bash -c '
subscription-manager register --org="$RH_ORG_ID" --activationkey="$RH_ACTIVATION_KEY" >/dev/null
echo "Enabled repositories:"
subscription-manager repos --list-enabled | grep "Repo ID:" | sed "s/Repo ID:/  -/"
'
}

generate_lockfile() {
    local base_image="$1"
    local input_file="$2"
    local output_file="$3"
    local repo_file="$4"

    print_status "Generating RPM lockfile with RHEL subscription..."
    print_status "Base image: $base_image"
    print_status "Input file: $input_file"
    print_status "Output file: $output_file"
    print_status "Repo file: $repo_file"

    if [[ -f "$output_file" ]]; then
        local backup_file
        backup_file="${output_file}.backup.$(date +%Y%m%d-%H%M%S)"
        cp "$output_file" "$backup_file"
        print_warning "Backed up existing $output_file to $backup_file"
    fi

    print_status "Running rpm-lockfile-prototype in registered container..."

    # Pass secrets via environment variables, not command line args
    if ! podman run --rm \
         -v "$(pwd):/work:Z" \
         -e "RH_ORG_ID=${org_id}" \
         -e "RH_ACTIVATION_KEY=${activation_key}" \
         -e "BASE_IMAGE=${base_image}" \
         -e "INPUT_FILE=${input_file}" \
         -e "OUTPUT_FILE=${output_file}" \
         -e "REPO_FILE=${repo_file}" \
         -e "RPM_LOCKFILE_VERSION=${rpm_lockfile_version}" \
         registry.access.redhat.com/ubi9 \
         bash -c '
set -euo pipefail

# Register with activation key (values from environment)
subscription-manager register --org="$RH_ORG_ID" --activationkey="$RH_ACTIVATION_KEY"

# Install dependencies
dnf install -y pip skopeo perl-interpreter

# Install rpm-lockfile-prototype
pip install --user "https://github.com/konflux-ci/rpm-lockfile-prototype/archive/refs/tags/${RPM_LOCKFILE_VERSION}.tar.gz"

# Copy repo file, filter to only enabled repos, and fix arch placeholder
perl -00 -ne '"'"'print if /^enabled\s*=\s*1$/m'"'"' /etc/yum.repos.d/redhat.repo > "/work/${REPO_FILE}"
sed -i "s/$(uname -m)/\$basearch/g" "/work/${REPO_FILE}"

# Generate lockfile
cd /work
~/.local/bin/rpm-lockfile-prototype --image "$BASE_IMAGE" --outfile "$OUTPUT_FILE" "$INPUT_FILE"
'; then
        print_error "Failed to generate lockfile"
        exit 1
    fi

    if [[ -f "$output_file" ]]; then
        print_success "RPM lockfile generated successfully: $output_file"

        local package_count
        package_count=$(grep -c "name:" "$output_file" || echo "0")
        print_status "Generated lockfile contains $package_count packages"

        if [[ $package_count -gt 0 ]]; then
            print_status "Sample packages in lockfile:"
            grep "name:" "$output_file" | head -5 | sed 's/^/  /'
            if [[ $package_count -gt 5 ]]; then
                print_status "  ... and $((package_count - 5)) more"
            fi
        fi
    else
        print_error "Lockfile was not generated"
        exit 1
    fi

    if [[ -f "$repo_file" ]]; then
        print_success "Repo file exported: $repo_file"
    fi
}

validate_lockfile() {
    local output_file="$1"

    print_status "Validating generated lockfile..."

    if ! grep -q "lockfileVersion:" "$output_file"; then
        print_error "Generated file does not appear to be a valid RPM lockfile"
        exit 1
    fi

    if ! grep -q "packages:" "$output_file"; then
        print_warning "Lockfile contains no packages - this may be expected if all packages are already in the base image"
    fi

    # Check for RHEL CDN URLs (not UBI)
    if grep -q "cdn.redhat.com" "$output_file"; then
        print_success "Lockfile contains RHEL CDN URLs (subscription content)"
    elif grep -q "cdn-ubi.redhat.com" "$output_file"; then
        print_warning "Lockfile contains only UBI URLs - subscription content may not be included"
    fi

    print_success "Lockfile validation passed"
}

# Parse command line arguments
base_image="$default_base_image"
input_file="$rpms_in_file"
output_file="$rpms_lock_file"
repo_file="$redhat_repo_file"
activation_key="${RH_ACTIVATION_KEY:-}"
org_id="${RH_ORG_ID:-}"
list_repos_only=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -k|--activation-key)
            activation_key="$2"
            shift 2
            ;;
        -O|--org)
            org_id="$2"
            shift 2
            ;;
        -i|--input)
            input_file="$2"
            shift 2
            ;;
        -o|--output)
            output_file="$2"
            shift 2
            ;;
        -r|--repo-file)
            repo_file="$2"
            shift 2
            ;;
        -b|--base-image)
            base_image="$2"
            shift 2
            ;;
        --list-repos)
            list_repos_only=true
            shift
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

if [[ "$list_repos_only" == "true" ]]; then
    # Minimal requirements check for --list-repos
    if ! command -v podman &> /dev/null; then
        print_error "podman is required but not installed"
        exit 1
    fi
    if [[ -z "${activation_key:-}" ]]; then
        print_error "Activation key is required. Use -k/--activation-key or set RH_ACTIVATION_KEY"
        exit 1
    fi
    if [[ -z "${org_id:-}" ]]; then
        print_error "Organisation ID is required. Use -O/--org or set RH_ORG_ID"
        exit 1
    fi
    list_repos
    exit 0
fi

print_status "Starting RPM lockfile generation with RHEL subscription..."
print_status "Working directory: $(pwd)"

check_requirements
generate_lockfile "$base_image" "$input_file" "$output_file" "$repo_file"
validate_lockfile "$output_file"

print_success "RPM lockfile generation completed!"
print_status "Next steps:"
print_status "  1. Review the generated $output_file"
print_status "  2. Commit $input_file, $output_file, and $repo_file to your repository"
print_status "  3. Create activation-key secret in Konflux namespace:"
print_status "     kubectl create secret generic activation-key -n <namespace> \\"
print_status "       --from-literal=org=<your-org-id> \\"
print_status "       --from-literal=activationkey=<your-activation-key>"
print_status "  4. Ensure your Tekton pipelines have the correct prefetch-input configuration"
