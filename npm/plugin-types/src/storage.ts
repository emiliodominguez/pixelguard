/**
 * Storage plugin types.
 *
 * Storage plugins handle where baseline screenshots are stored.
 * They can store baselines in cloud storage (S3, R2, Azure Blob, etc.)
 * instead of the local filesystem.
 */

/**
 * Input for storage read operation.
 */
export interface StorageReadInput {
	/** Relative path to the file */
	path: string;

	/** Plugin options from config */
	options: Record<string, unknown>;
}

/**
 * Input for storage write operation.
 */
export interface StorageWriteInput {
	/** Relative path to the file */
	path: string;

	/** Base64-encoded file data */
	data: string;

	/** Plugin options from config */
	options: Record<string, unknown>;
}

/**
 * Input for storage exists operation.
 */
export interface StorageExistsInput {
	/** Relative path to the file */
	path: string;

	/** Plugin options from config */
	options: Record<string, unknown>;
}

/**
 * Input for storage list operation.
 */
export interface StorageListInput {
	/** Directory path to list */
	path: string;

	/** Plugin options from config */
	options: Record<string, unknown>;
}

/**
 * Input for storage delete operation.
 */
export interface StorageDeleteInput {
	/** Relative path to the file */
	path: string;

	/** Plugin options from config */
	options: Record<string, unknown>;
}

/**
 * Storage plugin interface.
 *
 * Implement these hooks to create a custom storage backend.
 *
 * @example
 * ```typescript
 * import type { StoragePlugin } from '@pixelguard/plugin-types';
 * import { S3Client, GetObjectCommand, PutObjectCommand } from '@aws-sdk/client-s3';
 *
 * const client = new S3Client({ region: 'us-east-1' });
 *
 * export const read: StoragePlugin['read'] = async ({ path, options }) => {
 *   const command = new GetObjectCommand({
 *     Bucket: options.bucket as string,
 *     Key: path,
 *   });
 *   const response = await client.send(command);
 *   const body = await response.Body?.transformToByteArray();
 *   return Buffer.from(body!).toString('base64');
 * };
 *
 * export const write: StoragePlugin['write'] = async ({ path, data, options }) => {
 *   const command = new PutObjectCommand({
 *     Bucket: options.bucket as string,
 *     Key: path,
 *     Body: Buffer.from(data, 'base64'),
 *   });
 *   await client.send(command);
 * };
 * ```
 */
export interface StoragePlugin {
	/**
	 * Read a file from storage.
	 * @returns Base64-encoded file data
	 */
	read: (input: StorageReadInput) => Promise<string>;

	/**
	 * Write a file to storage.
	 */
	write: (input: StorageWriteInput) => Promise<void>;

	/**
	 * Check if a file exists in storage.
	 */
	exists: (input: StorageExistsInput) => Promise<boolean>;

	/**
	 * List files in a directory.
	 * @returns Array of file paths
	 */
	list: (input: StorageListInput) => Promise<string[]>;

	/**
	 * Delete a file from storage.
	 */
	delete: (input: StorageDeleteInput) => Promise<void>;
}
