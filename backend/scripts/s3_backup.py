#!/usr/bin/env python3
"""
S3 Backup Script for Baraba
Uses boto3 for S3-compatible storage operations
"""

import argparse
import json
import sys
import os

try:
    import boto3
    from botocore.config import Config
    from botocore.exceptions import ClientError, NoCredentialsError
except ImportError:
    print(json.dumps({
        "success": False,
        "error": "boto3 не е инсталиран. Инсталирайте с: pip install boto3"
    }))
    sys.exit(1)


def get_s3_client(endpoint: str, access_key: str, secret_key: str, region: str):
    """Create S3 client with given credentials"""
    config = Config(
        signature_version='s3v4',
        s3={'addressing_style': 'path'}
    )

    client_kwargs = {
        'service_name': 's3',
        'aws_access_key_id': access_key,
        'aws_secret_access_key': secret_key,
        'region_name': region,
        'config': config
    }

    if endpoint:
        client_kwargs['endpoint_url'] = endpoint

    return boto3.client(**client_kwargs)


def test_connection(endpoint: str, bucket: str, access_key: str, secret_key: str, region: str):
    """Test S3 connection"""
    try:
        client = get_s3_client(endpoint, access_key, secret_key, region)
        # Try to list objects (limited to 1) to verify access
        client.list_objects_v2(Bucket=bucket, MaxKeys=1)
        return {"success": True, "message": f"Връзката с S3 е успешна! Bucket: {bucket}"}
    except NoCredentialsError:
        return {"success": False, "message": "Невалидни credentials"}
    except ClientError as e:
        error_code = e.response.get('Error', {}).get('Code', '')
        if error_code == 'NoSuchBucket':
            return {"success": False, "message": f"Bucket '{bucket}' не съществува"}
        elif error_code == 'InvalidAccessKeyId':
            return {"success": False, "message": "Невалиден Access Key"}
        elif error_code == 'SignatureDoesNotMatch':
            return {"success": False, "message": "Невалиден Secret Key"}
        elif error_code == 'AccessDenied':
            return {"success": False, "message": "Достъпът е отказан. Проверете правата на credentials."}
        else:
            return {"success": False, "message": f"Грешка: {str(e)}"}
    except Exception as e:
        return {"success": False, "message": f"Грешка: {str(e)}"}


def upload_file(local_path: str, s3_key: str, endpoint: str, bucket: str,
                access_key: str, secret_key: str, region: str):
    """Upload file to S3"""
    try:
        if not os.path.exists(local_path):
            return {"success": False, "message": f"Файлът не съществува: {local_path}"}

        client = get_s3_client(endpoint, access_key, secret_key, region)
        client.upload_file(local_path, bucket, s3_key)
        return {"success": True, "message": "OK"}
    except ClientError as e:
        return {"success": False, "message": f"S3 upload грешка: {str(e)}"}
    except Exception as e:
        return {"success": False, "message": f"Грешка: {str(e)}"}


def list_backups(prefix: str, endpoint: str, bucket: str,
                 access_key: str, secret_key: str, region: str):
    """List backups from S3"""
    try:
        client = get_s3_client(endpoint, access_key, secret_key, region)

        response = client.list_objects_v2(Bucket=bucket, Prefix=prefix)

        backups = []
        for obj in response.get('Contents', []):
            key = obj['Key']
            filename = key.replace(prefix, '')

            # Only include baraba backups
            if 'baraba_backup' not in filename:
                continue

            size = obj['Size']
            last_modified = obj['LastModified'].strftime('%Y-%m-%d %H:%M:%S')

            backups.append({
                "key": key,
                "filename": filename,
                "size": size,
                "last_modified": last_modified
            })

        # Sort by last_modified descending
        backups.sort(key=lambda x: x['last_modified'], reverse=True)

        return {"success": True, "data": backups}
    except ClientError as e:
        return {"success": False, "error": str(e), "data": []}
    except Exception as e:
        return {"success": False, "error": str(e), "data": []}


def delete_backup(s3_key: str, endpoint: str, bucket: str,
                  access_key: str, secret_key: str, region: str):
    """Delete backup from S3"""
    try:
        client = get_s3_client(endpoint, access_key, secret_key, region)
        client.delete_object(Bucket=bucket, Key=s3_key)
        return {"success": True, "message": f"Backup изтрит успешно"}
    except ClientError as e:
        return {"success": False, "message": f"Грешка при изтриване: {str(e)}"}
    except Exception as e:
        return {"success": False, "message": f"Грешка: {str(e)}"}


def download_file(s3_key: str, local_path: str, endpoint: str, bucket: str,
                  access_key: str, secret_key: str, region: str):
    """Download file from S3"""
    try:
        client = get_s3_client(endpoint, access_key, secret_key, region)
        client.download_file(bucket, s3_key, local_path)
        return {"success": True, "message": "OK"}
    except ClientError as e:
        return {"success": False, "message": f"S3 download грешка: {str(e)}"}
    except Exception as e:
        return {"success": False, "message": f"Грешка: {str(e)}"}


def main():
    parser = argparse.ArgumentParser(description='S3 Backup Operations')
    parser.add_argument('action', choices=['test', 'upload', 'list', 'delete', 'download'])
    parser.add_argument('--endpoint', default='')
    parser.add_argument('--bucket', required=True)
    parser.add_argument('--access-key', required=True)
    parser.add_argument('--secret-key', required=True)
    parser.add_argument('--region', default='us-east-1')
    parser.add_argument('--prefix', default='backups/')
    parser.add_argument('--local-path', default='')
    parser.add_argument('--s3-key', default='')

    args = parser.parse_args()

    if args.action == 'test':
        result = test_connection(args.endpoint, args.bucket, args.access_key,
                                  args.secret_key, args.region)
    elif args.action == 'upload':
        if not args.local_path or not args.s3_key:
            result = {"success": False, "message": "local-path и s3-key са задължителни за upload"}
        else:
            result = upload_file(args.local_path, args.s3_key, args.endpoint,
                                 args.bucket, args.access_key, args.secret_key, args.region)
    elif args.action == 'list':
        result = list_backups(args.prefix, args.endpoint, args.bucket,
                              args.access_key, args.secret_key, args.region)
    elif args.action == 'delete':
        if not args.s3_key:
            result = {"success": False, "message": "s3-key е задължителен за delete"}
        else:
            result = delete_backup(args.s3_key, args.endpoint, args.bucket,
                                   args.access_key, args.secret_key, args.region)
    elif args.action == 'download':
        if not args.local_path or not args.s3_key:
            result = {"success": False, "message": "local-path и s3-key са задължителни за download"}
        else:
            result = download_file(args.s3_key, args.local_path, args.endpoint,
                                   args.bucket, args.access_key, args.secret_key, args.region)
    else:
        result = {"success": False, "message": f"Непозната команда: {args.action}"}

    print(json.dumps(result, ensure_ascii=False))


if __name__ == '__main__':
    main()
