# GCP sccache Setup for GitHub Actions CI

This guide explains how to configure Google Cloud Storage as a cache backend for sccache in GitHub Actions.

## Prerequisites

- Google Cloud project with billing enabled
- `gcloud` CLI installed and authenticated
- Admin access to the GitHub repository

## GCP Setup

### 1. Create a GCS Bucket

```bash
# Replace YOUR-BUCKET-NAME with your desired bucket name
gcloud storage buckets create gs://YOUR-BUCKET-NAME --location=us-central1

# Optional: Set lifecycle policy to auto-delete old cache entries (e.g., 30 days)
cat > /tmp/lifecycle.json << 'EOF'
{
  "rule": [
    {
      "action": {"type": "Delete"},
      "condition": {"age": 30}
    }
  ]
}
EOF
gcloud storage buckets update gs://YOUR-BUCKET-NAME --lifecycle-file=/tmp/lifecycle.json
```

### 2. Set Up Workload Identity Federation

Workload Identity Federation allows GitHub Actions to authenticate to GCP without service account keys.

```bash
# Set variables
export PROJECT_ID="your-gcp-project-id"
export PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
export GITHUB_ORG="your-github-org"
export GITHUB_REPO="your-repo-name"

# Create a Workload Identity Pool
gcloud iam workload-identity-pools create "github-actions" \
  --project="$PROJECT_ID" \
  --location="global" \
  --display-name="GitHub Actions"

# Create a provider for GitHub
gcloud iam workload-identity-pools providers create-oidc "github" \
  --project="$PROJECT_ID" \
  --location="global" \
  --workload-identity-pool="github-actions" \
  --display-name="GitHub" \
  --attribute-mapping="google.subject=assertion.sub,attribute.repository=assertion.repository" \
  --issuer-uri="https://token.actions.githubusercontent.com"

# Create a service account for sccache
gcloud iam service-accounts create sccache-ci \
  --project="$PROJECT_ID" \
  --display-name="sccache CI Service Account"

# Grant the service account access to the GCS bucket
gcloud storage buckets add-iam-policy-binding gs://YOUR-BUCKET-NAME \
  --member="serviceAccount:sccache-ci@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/storage.objectAdmin"

# Allow GitHub Actions to impersonate the service account
gcloud iam service-accounts add-iam-policy-binding \
  sccache-ci@${PROJECT_ID}.iam.gserviceaccount.com \
  --project="$PROJECT_ID" \
  --role="roles/iam.workloadIdentityUser" \
  --member="principalSet://iam.googleapis.com/projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions/attribute.repository/${GITHUB_ORG}/${GITHUB_REPO}"
```

### 3. Get the Workload Identity Provider Resource Name

```bash
echo "projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/github-actions/providers/github"
```

## GitHub Repository Configuration

### Add Repository Variables

Navigate to **Settings → Secrets and variables → Actions → Variables** and add:

| Variable | Value | Description |
|----------|-------|-------------|
| `SCCACHE_GCS_BUCKET` | `your-bucket-name` | GCS bucket name (without `gs://` prefix) |
| `GCP_WORKLOAD_IDENTITY_PROVIDER` | `projects/PROJECT_NUMBER/locations/global/workloadIdentityPools/github-actions/providers/github` | Full resource name of the WIF provider |
| `GCP_SERVICE_ACCOUNT` | `sccache-ci@PROJECT_ID.iam.gserviceaccount.com` | Service account email |

## CI Workflow Configuration

The workflow (`.github/workflows/ci.yml`) is already configured to use these variables:

```yaml
permissions:
  actions: read
  contents: read
  id-token: write  # Required for Workload Identity Federation

jobs:
  main:
    runs-on: ubuntu-latest
    env:
      CARGO_INCREMENTAL: 0
      RUSTC_WRAPPER: sccache
      SCCACHE_GCS_BUCKET: ${{ vars.SCCACHE_GCS_BUCKET }}
      SCCACHE_GCS_RW_MODE: READ_WRITE
      SCCACHE_GCS_KEY_PREFIX: sccache
    steps:
      # ... checkout and other steps ...

      - name: Authenticate to Google Cloud
        uses: google-github-actions/auth@v2
        with:
          workload_identity_provider: ${{ vars.GCP_WORKLOAD_IDENTITY_PROVIDER }}
          service_account: ${{ vars.GCP_SERVICE_ACCOUNT }}

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.9
```

## Verifying the Setup

After running the CI workflow, you can verify the cache is working:

1. Check the sccache stats in the workflow logs
2. Verify objects are created in the GCS bucket:
   ```bash
   gcloud storage ls gs://YOUR-BUCKET-NAME/sccache/
   ```

## Troubleshooting

### Authentication Errors

- Ensure `id-token: write` permission is set in the workflow
- Verify the Workload Identity Pool and Provider are correctly configured
- Check that the service account has the correct IAM binding

### Cache Not Being Used

- Verify `SCCACHE_GCS_RW_MODE` is set to `READ_WRITE`
- Check that the service account has `roles/storage.objectAdmin` on the bucket
- Ensure `RUSTC_WRAPPER: sccache` is set in the environment

### Bucket Access Denied

- Confirm the bucket name is correct (no `gs://` prefix in the variable)
- Verify the service account email is correct
- Check IAM permissions on the bucket

## Cost Considerations

- GCS storage costs are minimal for cache data (~$0.02/GB/month for Standard storage)
- Consider setting a lifecycle policy to auto-delete old cache entries
- Network egress from GCP to GitHub Actions runners is free within the same region
