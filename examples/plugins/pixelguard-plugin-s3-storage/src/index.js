/**
 * Pixelguard S3 Storage Plugin
 *
 * Stores baseline screenshots in AWS S3 instead of the local filesystem.
 * Useful for sharing baselines across teams and CI environments.
 *
 * Configuration:
 * {
 *   "plugins": ["pixelguard-plugin-s3-storage"],
 *   "pluginOptions": {
 *     "pixelguard-plugin-s3-storage": {
 *       "bucket": "my-pixelguard-baselines",
 *       "region": "us-east-1",
 *       "prefix": "baselines/"
 *     }
 *   }
 * }
 */

const { S3Client, GetObjectCommand, PutObjectCommand, HeadObjectCommand, ListObjectsV2Command, DeleteObjectCommand } = require("@aws-sdk/client-s3");

let s3Client = null;

/**
 * Get or create S3 client instance.
 *
 * @param {Object} options - Plugin options
 * @returns {S3Client}
 */
function getClient(options) {
	if (!s3Client) {
		s3Client = new S3Client({
			region: options.region || "us-east-1",
			// Credentials are automatically loaded from environment or IAM role
		});
	}
	return s3Client;
}

/**
 * Get the full S3 key for a path.
 *
 * @param {string} path - Relative path
 * @param {Object} options - Plugin options
 * @returns {string}
 */
function getKey(path, options) {
	const prefix = options.prefix || "";
	return prefix + path;
}

/**
 * Read a file from S3.
 *
 * @param {Object} input - Storage input
 * @param {string} input.path - Path to read
 * @param {Object} input.options - Plugin options
 * @returns {Promise<{data: string}>} Base64 encoded data
 */
async function read(input) {
	const { path, options } = input;

	if (!options.bucket) {
		throw new Error("S3 bucket is required. Set it in pluginOptions.pixelguard-plugin-s3-storage.bucket");
	}

	const client = getClient(options);
	const key = getKey(path, options);

	try {
		const response = await client.send(
			new GetObjectCommand({
				Bucket: options.bucket,
				Key: key,
			}),
		);

		// Convert stream to buffer
		const chunks = [];
		for await (const chunk of response.Body) {
			chunks.push(chunk);
		}
		const buffer = Buffer.concat(chunks);

		return {
			data: buffer.toString("base64"),
		};
	} catch (error) {
		if (error.name === "NoSuchKey") {
			throw new Error(`File not found in S3: ${key}`);
		}
		throw error;
	}
}

/**
 * Write a file to S3.
 *
 * @param {Object} input - Storage input
 * @param {string} input.path - Path to write
 * @param {string} input.data - Base64 encoded data
 * @param {Object} input.options - Plugin options
 * @returns {Promise<void>}
 */
async function write(input) {
	const { path, data, options } = input;

	if (!options.bucket) {
		throw new Error("S3 bucket is required. Set it in pluginOptions.pixelguard-plugin-s3-storage.bucket");
	}

	const client = getClient(options);
	const key = getKey(path, options);
	const buffer = Buffer.from(data, "base64");

	// Determine content type from extension
	let contentType = "application/octet-stream";
	if (path.endsWith(".png")) {
		contentType = "image/png";
	} else if (path.endsWith(".jpg") || path.endsWith(".jpeg")) {
		contentType = "image/jpeg";
	}

	await client.send(
		new PutObjectCommand({
			Bucket: options.bucket,
			Key: key,
			Body: buffer,
			ContentType: contentType,
		}),
	);
}

/**
 * Check if a file exists in S3.
 *
 * @param {Object} input - Storage input
 * @param {string} input.path - Path to check
 * @param {Object} input.options - Plugin options
 * @returns {Promise<{exists: boolean}>}
 */
async function exists(input) {
	const { path, options } = input;

	if (!options.bucket) {
		throw new Error("S3 bucket is required. Set it in pluginOptions.pixelguard-plugin-s3-storage.bucket");
	}

	const client = getClient(options);
	const key = getKey(path, options);

	try {
		await client.send(
			new HeadObjectCommand({
				Bucket: options.bucket,
				Key: key,
			}),
		);
		return { exists: true };
	} catch (error) {
		if (error.name === "NotFound") {
			return { exists: false };
		}
		throw error;
	}
}

/**
 * List files in S3 under a prefix.
 *
 * @param {Object} input - Storage input
 * @param {string} input.path - Path prefix to list
 * @param {Object} input.options - Plugin options
 * @returns {Promise<{files: string[]}>}
 */
async function list(input) {
	const { path, options } = input;

	if (!options.bucket) {
		throw new Error("S3 bucket is required. Set it in pluginOptions.pixelguard-plugin-s3-storage.bucket");
	}

	const client = getClient(options);
	const prefix = getKey(path, options);

	const files = [];
	let continuationToken;

	do {
		const response = await client.send(
			new ListObjectsV2Command({
				Bucket: options.bucket,
				Prefix: prefix,
				ContinuationToken: continuationToken,
			}),
		);

		if (response.Contents) {
			for (const obj of response.Contents) {
				// Remove the prefix to get relative path
				const relativePath = obj.Key.slice((options.prefix || "").length);
				files.push(relativePath);
			}
		}

		continuationToken = response.NextContinuationToken;
	} while (continuationToken);

	return { files };
}

/**
 * Delete a file from S3.
 *
 * @param {Object} input - Storage input
 * @param {string} input.path - Path to delete
 * @param {Object} input.options - Plugin options
 * @returns {Promise<void>}
 */
async function deleteFile(input) {
	const { path, options } = input;

	if (!options.bucket) {
		throw new Error("S3 bucket is required. Set it in pluginOptions.pixelguard-plugin-s3-storage.bucket");
	}

	const client = getClient(options);
	const key = getKey(path, options);

	await client.send(
		new DeleteObjectCommand({
			Bucket: options.bucket,
			Key: key,
		}),
	);
}

module.exports = {
	read,
	write,
	exists,
	list,
	delete: deleteFile,
};
