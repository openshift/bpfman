# Hermetic Build Comparison: lightspeed-service vs bpfman

## Overview
Comparison of cachi2 hermetic build implementations between openshift/lightspeed-service (working) and openshift/bpfman (experiencing issues).

---

## 1. Containerfile Differences

### lightspeed-service Containerfile
- **File**: `/tmp/lightspeed-service/Containerfile`
- **Key Features**:
  - Uses `ARG HERMETIC=false` with conditional logic
  - Conditional RPM installation based on HERMETIC flag:
    ```dockerfile
    RUN if [ "$HERMETIC" == "true" ]; then \
          microdnf install -y --nodocs --setopt=keepcache=0 --setopt=tsflags=nodocs openshift-clients; \
        else \
          # Downloads from external source
        fi
    ```
  - Base package installation happens BEFORE hermetic check:
    ```dockerfile
    RUN microdnf install -y --nodocs --setopt=keepcache=0 --setopt=tsflags=nodocs \
        python3.11 python3.11-devel python3.11-pip
    ```
  - Uses microdnf (not dnf)
  - Multi-stage build with platform-specific FROM:
    - `FROM --platform=$BUILDPLATFORM registry.redhat.io/ubi9/ubi-minimal:latest`

### bpfman Containerfile
- **File**: `/home/aim/src/github.com/openshift/bpfman/Containerfile.bpfman.openshift`
- **Key Features**:
  - Uses `ARG DNF_CMD="dnf"` variable
  - All package installation uses DNF_CMD:
    ```dockerfile
    RUN ${DNF_CMD} update -y \
        && ${DNF_CMD} install -y gcc openssl-devel cmake clang-devel rust cargo \
        && ${DNF_CMD} clean all
    ```
  - Uses full UBI9 base: `FROM registry.access.redhat.com/ubi9/ubi:latest`
  - No conditional logic for hermetic vs non-hermetic builds
  - Multi-stage build pattern

**CRITICAL DIFFERENCE**: lightspeed-service uses microdnf with explicit flags, while bpfman uses dnf via ARG substitution.

---

## 2. Pipeline Configuration

### lightspeed-service Pipeline
- **Files**:
  - `.tekton/lightspeed-service-pull-request.yaml`
  - `.tekton/lightspeed-service-push.yaml`

**Key Configuration**:
```yaml
params:
  - name: hermetic
    value: 'true'
  - name: prefetch-input
    value: '[{"type": "rpm", "path": "."}, {"type": "pip", "path": ".", "allow_binary": "true"}]'
  - name: build-args-file
    value: build.args
  - name: build-platforms
    value:
    - linux/x86_64
    - linux-c6gd2xlarge/arm64
```

**Resource Allocation**:
```yaml
taskRunSpecs:
  - computeResources:
      limits:
        memory: 10Gi
      requests:
        memory: 10Gi
    pipelineTaskName: prefetch-dependencies
  - computeResources:
      limits:
        memory: 12Gi
      requests:
        memory: 12Gi
    pipelineTaskName: build-images
```

### bpfman Pipeline
- **File**: `.tekton/bpfman-daemon-ystream-push.yaml`

**Key Configuration**:
```yaml
params:
  - name: hermetic
    value: 'true'
  - name: prefetch-input
    value: '[{"type": "rpm", "path": "."}, {"type": "cargo", "path": "."}]'
  - name: build-args-file
    value: OPENSHIFT-VERSION
  - name: build-platforms
    value:
    - linux/x86_64
    - linux/s390x
    - linux/arm64
    - linux/ppc64le
```

**Pipeline Reference**:
```yaml
pipelineRef:
  name: build-pipeline  # Uses a reference, not inline spec
```

**CRITICAL DIFFERENCE**:
1. lightspeed-service has INLINE pipelineSpec with all task details
2. lightspeed-service specifies resource limits for prefetch and build tasks
3. bpfman uses a pipeline reference without resource specs
4. Different build platforms (bpfman includes s390x and ppc64le)

---

## 3. RPM Lock Files

### lightspeed-service rpms.in.yaml
```yaml
packages: [python3.11, python3.11-devel, python3.11-pip, tar, gzip, openshift-clients]
contentOrigin:
  repofiles: ["./ubi.repo", "./redhat.repo"]
```

**Notable**:
- Simple package list (no context or arches specified)
- References TWO repo files: ubi.repo AND redhat.repo
- redhat.repo is 337KB (very large, contains ALL Red Hat repos)
- openshift-clients comes from rhocp-4.17 repo

### bpfman rpms.in.yaml
```yaml
packages:
  - gcc
  - openssl-devel
  - cmake
  - clang-devel
  - rust
  - cargo
contentOrigin:
  repofiles:
    - ./ubi.repo
arches:
  - x86_64
  - s390x
  - aarch64
  - ppc64le
context:
  containerfile:
    file: ./Containerfile.bpfman.openshift
    stageName: bpfman-build
```

**Notable**:
- Explicit architecture list (4 arches)
- Only references ubi.repo (no redhat.repo)
- Has context section pointing to specific Containerfile stage
- More structured YAML format

**CRITICAL DIFFERENCE**:
1. lightspeed-service includes redhat.repo with entitled content
2. bpfman only uses public UBI repos
3. lightspeed-service has arch-specific packages in rpms.lock.yaml
4. bpfman specifies arches in rpms.in.yaml

---

## 4. Repository Files

### lightspeed-service ubi.repo
```ini
[ubi-9-for-$basearch-baseos-rpms]
baseurl = https://cdn-ubi.redhat.com/content/public/ubi/dist/ubi9/9/$basearch/baseos/os
enabled = 1
gpgkey = file:///etc/pki/rpm-gpg/RPM-GPG-KEY-redhat-release
gpgcheck = 1
```
- Uses $basearch variable
- Has gpgcheck = 1
- Includes baseos, appstream, and codeready-builder repos

### bpfman ubi.repo
```ini
[ubi-9-for-$basearch-baseos-rpms]
baseurl = https://cdn-ubi.redhat.com/content/public/ubi/dist/ubi9/9/$arch/baseos/os
enabled = 1
gpgcheck = 0
skip_if_unavailable = False
```
- Uses $arch variable (NOT $basearch)
- Has gpgcheck = 0
- Has skip_if_unavailable = False
- Same three repos: baseos, appstream, codeready-builder

### lightspeed-service redhat.repo
- 337KB file containing all Red Hat subscription repos
- Includes rhocp-4.17 repo:
```ini
[rhocp-4.17-for-rhel-9-$basearch-rpms]
name = Red Hat OpenShift Container Platform 4.17 for RHEL 9 $basearch (RPMs)
baseurl = https://cdn.redhat.com/content/dist/layered/rhel9/$basearch/rhocp/4.17/os
enabled = 1
```
- Uses SSL certificates for authentication

**CRITICAL DIFFERENCE**:
1. Variable naming: $basearch vs $arch
2. GPG checking: enabled vs disabled
3. lightspeed-service has entitled repos, bpfman doesn't

---

## 5. Build Arguments

### lightspeed-service build.args
```
HERMETIC=true
```
- Simple single argument file
- Sets HERMETIC build arg to true

### bpfman OPENSHIFT-VERSION
```
BUILDVERSION=0.5.11
```
- Sets version build arg
- Used in labels

**CRITICAL DIFFERENCE**: Different build arg purposes and names

---

## 6. Multi-Architecture Build Patterns

### lightspeed-service
- Uses matrix strategy in pipeline with PLATFORM parameter
- Platforms specified at pipeline level:
  ```yaml
  - linux/x86_64
  - linux-c6gd2xlarge/arm64  # Special notation for ARM
  ```
- Uses buildah-remote-oci-ta task
- Creates image index after building all platforms

### bpfman
- Similar matrix strategy
- Four platforms:
  ```yaml
  - linux/x86_64
  - linux/s390x
  - linux/arm64
  - linux/ppc64le
  ```
- Uses pipeline reference instead of inline spec

---

## 7. Package Manager Handling

### lightspeed-service Approach
1. Uses microdnf (minimal DNF)
2. Explicit flags: `--nodocs --setopt=keepcache=0 --setopt=tsflags=nodocs`
3. Conditional logic for hermetic vs non-hermetic
4. Packages installed in specific order:
   - First: python packages (always)
   - Then: conditional openshift-clients

### bpfman Approach
1. Uses dnf via ARG substitution: `ARG DNF_CMD="dnf"`
2. Single RUN command with && chaining
3. Includes update: `${DNF_CMD} update -y`
4. Cleans up: `${DNF_CMD} clean all`
5. No conditional logic

**CRITICAL ISSUE**: The DNF_CMD substitution might not work correctly with cachi2's injection mechanism.

---

## 8. Prefetch Configuration

### lightspeed-service
```yaml
prefetch-input: '[{"type": "rpm", "path": "."}, {"type": "pip", "path": ".", "allow_binary": "true"}]'
```
- Prefetches both RPMs and Python packages
- Allows binary wheels for pip

### bpfman
```yaml
prefetch-input: '[{"type": "rpm", "path": "."}, {"type": "cargo", "path": "."}]'
```
- Prefetches RPMs and Cargo crates
- No allow_binary flag

---

## Key Findings and Recommendations

### Issues Identified in bpfman Setup

1. **DNF_CMD Variable Usage**:
   - Cachi2 might inject its wrapper as `microdnf` or modify PATH
   - Using `${DNF_CMD}` might bypass cachi2's injection
   - **Recommendation**: Use direct `microdnf` or `dnf` commands

2. **Repository File Variable Mismatch**:
   - bpfman uses `$arch` while lightspeed-service uses `$basearch`
   - This could cause architecture resolution issues
   - **Recommendation**: Change to `$basearch` for consistency

3. **GPG Check Disabled**:
   - bpfman has `gpgcheck = 0`
   - Could indicate previous issues with signature verification
   - **Recommendation**: Investigate why GPG check was disabled

4. **Missing Resource Limits**:
   - bpfman pipeline doesn't specify resource limits for tasks
   - Could cause OOM issues during prefetch
   - **Recommendation**: Add explicit memory limits like lightspeed-service

5. **Pipeline Reference vs Inline Spec**:
   - bpfman uses `pipelineRef: build-pipeline`
   - Might be using older pipeline version without proper cachi2 support
   - **Recommendation**: Check pipeline version or use inline spec

6. **Architecture Specification**:
   - bpfman specifies arches in rpms.in.yaml
   - lightspeed-service doesn't, but has arch-specific entries in lock file
   - Might cause issues with lock file generation
   - **Recommendation**: Ensure arches match between files

### What lightspeed-service Does Right

1. Uses microdnf with explicit flags
2. Has conditional logic for hermetic builds
3. Includes comprehensive redhat.repo for entitled content
4. Specifies resource limits in pipeline
5. Uses inline pipeline spec with full control
6. Has arch-specific lock file entries
7. Higher memory allocation (10-12Gi)

### Recommended Changes for bpfman

1. **Containerfile**:
   - Change `ARG DNF_CMD="dnf"` approach to direct commands
   - Add HERMETIC arg with conditional logic
   - Use microdnf instead of dnf for consistency

2. **ubi.repo**:
   - Change `$arch` to `$basearch`
   - Consider enabling gpgcheck if possible

3. **Pipeline**:
   - Add resource limits for prefetch-dependencies (10Gi)
   - Add resource limits for build-images (12Gi)
   - Consider using inline pipelineSpec for better control

4. **rpms.in.yaml**:
   - Consider if entitled repos are needed
   - Ensure arch list matches build platforms

5. **Lock File Generation**:
   - Document the process for updating rpms.lock.yaml
   - Ensure it's generated with cachi2 tooling
