# AWS S3: Setup

A WebAssembly (WASM) plugin for secure command and control (C2) communication using AWS S3. 
This plugin enables bidirectional communication between C2 servers and agents through S3 bucket operations.
This guide covers the complete AWS S3 configuration required for the C2 communication plugin.

## Prerequisites

- AWS Account with billing enabled
- AWS CLI installed and configured (optional but recommended)
- Appropriate permissions to create S3 buckets and IAM users


## Step 1: Create S3 Bucket

### Using AWS Console

1. Navigate to the S3 service in AWS Console
2. Click "Create bucket"
3. Choose a bucket name that blends with your operational environment
4. Select an appropriate AWS region for your operations
5. Leave all other settings as default
6. Click "Create bucket"

### Using AWS CLI

```bash
# Replace with your chosen bucket name and region
aws s3 mb s3://your-c4-bucket --region us-east-1
```


## Step 2: Configure Bucket Policy

#### Bucket Policy

Apply the following bucket policy to allow full S3 operations. Replace `your-c4-bucket` with your actual bucket name:

Navigate to: S3 Console → Your Bucket → Permissions → Bucket Policy

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": "s3:*",
            "Resource": [
                "arn:aws:s3:::your-c4-bucket",
                "arn:aws:s3:::your-c4-bucket/*"
            ]
        }
    ]
}
```

#### Block Public Access

Ensure "Block all public access" is **enabled** (this should be the default). Your bucket should not be publicly accessible.


## Step 3: Create IAM User for C2 Operations

### Using AWS Console

#### Navigate to IAM service

1. Click "Users" → "Add users"
2. Enter username: c2-operations (or similar operational name)
3. Select "Programmatic access"
4. Click "Next: Permissions"
5. Click "Attach existing policies directly"
6. Click "Create policy"

#### IAM Policy

Create a custom policy with the following JSON (replace `your-c4-bucket`):

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
                "arn:aws:s3:::your-c4-bucket",
                "arn:aws:s3:::your-c4-bucket/*"
            ]
        }
    ]
}
```

### Using AWS CLI

```bash
# Create the IAM user
aws iam create-user --user-name c4

# Create access keys (save the output securely)
aws iam create-access-key --user-name c4

# Create the policy file (save the JSON above as c4-policy.json)
aws iam create-policy --policy-name C4BucketAccess --policy-document file://c4-policy.json

# Attach the policy to the user
aws iam attach-user-policy --user-name c4 --policy-arn arn:aws:iam::YOUR-ACCOUNT-ID:policy/C2BucketAccess
```


## Step 4: Configure Credentials

After creating the IAM user, you'll receive:

* **Access Key ID:** AKIA... (public identifier)
* **Secret Access Key:** Long secret string (keep secure)

Add the credentials to your `~/.aws/credentials` file as a new profile

```bash
[c4]
aws_access_key_id = AKIA...
aws_secret_access_key = supersecretaccesskey
```


## Step 5: Test Configuration

#### Verify Bucket Access

Test your configuration using the AWS CLI:

```bash
# Test bucket listing
aws s3 ls s3://your-c4-bucket --profile c4

# Test file upload
echo "test" | aws s3 cp - s3://your-c4-bucket/test.txt --profile c4
# Test file download
aws s3 cp s3://your-c4-bucket/test.txt - --profile c4

# Test file deletion
aws s3 rm s3://your-c4-bucket/test.txt --profile c4
```


## Cleanup

When finished with the S3 bucket for C4 operations, use the AWS CLI to cleanup:

```bash
# Delete all objects in bucket
aws s3 rm s3://your-c4-bucket --recursive

# Delete the bucket
aws s3 rb s3://your-c4-bucket

# Delete IAM user access keys
aws iam delete-access-key --user-name c4 --access-key-id AKIA...

# Delete IAM user
aws iam delete-user --user-name c4
```