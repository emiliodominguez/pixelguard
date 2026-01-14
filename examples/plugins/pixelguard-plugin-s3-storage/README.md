# pixelguard-plugin-s3-storage

A Pixelguard storage plugin that stores baseline screenshots in AWS S3.

## Why Use S3 Storage?

- **Share baselines across teams**: Everyone uses the same baselines from S3
- **CI/CD friendly**: No need to commit large binary files to git
- **Version history**: S3 versioning can track baseline changes over time
- **Scalable**: Handle thousands of screenshots without bloating your repo

## Installation

```bash
npm install pixelguard-plugin-s3-storage @aws-sdk/client-s3
```

## Configuration

Add the plugin to your `pixelguard.config.json`:

```json
{
  "source": "storybook",
  "baseUrl": "http://localhost:6006",
  "plugins": ["pixelguard-plugin-s3-storage"],
  "pluginOptions": {
    "pixelguard-plugin-s3-storage": {
      "bucket": "my-pixelguard-baselines",
      "region": "us-east-1",
      "prefix": "baselines/"
    }
  }
}
```

### Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `bucket` | `string` | Yes | - | S3 bucket name |
| `region` | `string` | No | `us-east-1` | AWS region |
| `prefix` | `string` | No | `""` | Key prefix for all files |

## AWS Credentials

The plugin uses the AWS SDK's default credential provider chain:

1. Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
2. Shared credentials file (`~/.aws/credentials`)
3. IAM role (when running on EC2, ECS, Lambda, etc.)

### Local Development

Set environment variables:

```bash
export AWS_ACCESS_KEY_ID="your-access-key"
export AWS_SECRET_ACCESS_KEY="your-secret-key"
export AWS_REGION="us-east-1"
```

Or use AWS CLI to configure:

```bash
aws configure
```

### CI/CD (GitHub Actions)

```yaml
- name: Run visual tests
  env:
    AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
    AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
    AWS_REGION: us-east-1
  run: npx pixelguard test --ci
```

### CI/CD (GitLab CI with OIDC)

```yaml
visual-tests:
  id_tokens:
    AWS_TOKEN:
      aud: https://gitlab.com
  script:
    - npx pixelguard test --ci
```

## S3 Bucket Setup

### Create Bucket

```bash
aws s3 mb s3://my-pixelguard-baselines --region us-east-1
```

### Bucket Policy (Optional)

For CI/CD access, create an IAM policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject",
        "s3:DeleteObject",
        "s3:ListBucket"
      ],
      "Resource": [
        "arn:aws:s3:::my-pixelguard-baselines",
        "arn:aws:s3:::my-pixelguard-baselines/*"
      ]
    }
  ]
}
```

### Enable Versioning (Recommended)

```bash
aws s3api put-bucket-versioning \
  --bucket my-pixelguard-baselines \
  --versioning-configuration Status=Enabled
```

## Storage Structure

With default configuration, files are stored as:

```
s3://my-pixelguard-baselines/
├── baselines/
│   ├── button--primary.png
│   ├── button--secondary.png
│   ├── card--default.png
│   └── ...
```

## Plugin Development

This plugin demonstrates:

- **Storage plugin interface**: Implementing all 5 storage hooks
- **AWS SDK integration**: Using `@aws-sdk/client-s3`
- **Streaming data**: Handling large files efficiently
- **Error handling**: Proper error messages for common issues

### Storage Hooks

```javascript
// Read baseline from S3
async function read({ path, options }) {
  const response = await s3.send(new GetObjectCommand({
    Bucket: options.bucket,
    Key: path,
  }));
  return { data: buffer.toString('base64') };
}

// Write baseline to S3
async function write({ path, data, options }) {
  await s3.send(new PutObjectCommand({
    Bucket: options.bucket,
    Key: path,
    Body: Buffer.from(data, 'base64'),
  }));
}

// Check if baseline exists
async function exists({ path, options }) {
  try {
    await s3.send(new HeadObjectCommand({ ... }));
    return { exists: true };
  } catch {
    return { exists: false };
  }
}

// List all baselines
async function list({ path, options }) {
  const response = await s3.send(new ListObjectsV2Command({ ... }));
  return { files: response.Contents.map(obj => obj.Key) };
}

// Delete a baseline
async function deleteFile({ path, options }) {
  await s3.send(new DeleteObjectCommand({ ... }));
}
```

## Migrating from Local Storage

To migrate existing baselines to S3:

```bash
# Upload existing baselines
aws s3 sync .pixelguard/baselines/ s3://my-bucket/baselines/

# Add plugin to config
# Then remove local baselines from git (optional)
git rm -r --cached .pixelguard/baselines/
```

## Troubleshooting

### "Access Denied" Error

- Check IAM permissions
- Verify bucket name and region
- Ensure credentials are configured

### "Bucket does not exist" Error

- Create the bucket first
- Check for typos in bucket name
- Verify the region matches

### Slow Performance

- Use a region close to your CI runners
- Consider S3 Transfer Acceleration for global teams

## License

MIT
